use std::sync::Arc;

use actix::{fut::WrapFuture, Actor, Addr, AsyncContext, Context, Handler, Message, Recipient};
use gdcf::{
    api::request::level::{LevelRequestType, LevelsRequest, SearchFilters},
    cache::CacheEntry,
    future::CloneCached,
    Gdcf,
};
use gdcf_diesel::{Cache, Entry};
use gdcf_model::{
    level::{DemonRating, Level, LevelRating, PartialLevel},
    song::NewgroundsSong,
    user::Creator,
};
use gdrs::BoomlingsClient;
use log::{debug, error, info, warn};
use reqwest::r#async::Client;
use tokio::{
    self,
    prelude::future::{result, Either, Future},
};

use crate::{actor::database::DeleteMessage, model::demonlist::record::Record};

/// Actor for whatever the fuck just happens to need to be done and isn't database access
#[allow(missing_debug_implementations)]
pub struct HttpActor {
    pub(super) gdcf: Gdcf<BoomlingsClient, Cache>,
    pub(super) http_client: Client,
    pub(super) discord_webhook_url: Arc<Option<String>>,
    pub(super) deletor: Recipient<DeleteMessage<i32, Record>>,
}

impl HttpActor {
    pub fn from_env(deletor: Recipient<DeleteMessage<i32, Record>>) -> Addr<Self> {
        info!("Initalizing HttpActor from environment data");

        let gdcf_url = std::env::var("GDCF_DATABASE_URL").expect("GDCF_DATABASE_URL is not set");

        let cache = Cache::postgres(gdcf_url).expect("haha is break");
        let client = BoomlingsClient::new();

        cache.initialize().unwrap();

        HttpActor {
            deletor,
            gdcf: Gdcf::new(client, cache),
            http_client: Client::builder()
                .build()
                .expect("Failed to create reqwest client"),
            discord_webhook_url: Arc::new(std::env::var("DISCORD_WEBHOOK").ok()),
        }
        .start()
    }

    pub fn execute_discord_webhook(
        &self,
        data: serde_json::Value,
    ) -> impl Future<Item = (), Error = ()> {
        if let Some(ref uri) = *self.discord_webhook_url {
            info!("Executing discord webhook!");

            let future = self
                .http_client
                .post(uri)
                .header("Content-Type", "application/json")
                .body(data.to_string())
                .send()
                .map_err(move |error| {
                    error!(
                        "INTERNAL SERVER ERROR: Failure to execute discord webhook: {:?}",
                        error
                    )
                })
                .map(|_| debug!("Successfully executed discord webhook"));

            Either::A(future)
        } else {
            warn!("Trying to execute webhook, though no link was configured!");

            Either::B(result(Ok(())))
        }
    }

    /// Creates a future that resolves to `()` if a `HEAD` request to the given URL receives a
    /// non-error response status code.
    pub fn if_exists(&self, url: &str) -> impl Future<Item = (), Error = ()> {
        debug!(
            "Verifying {} response to HEAD request with successful status code",
            url
        );

        self.http_client
            .head(url)
            .send()
            .map_err(|error| error!("INTERNAL SERVER ERROR: HEAD request failed: {:?}", error))
            .and_then(|response| {
                let status = response.status().as_u16();

                if 200 <= status && status < 400 {
                    Ok(())
                } else {
                    Err(())
                }
            })
    }
}

impl Actor for HttpActor {
    type Context = Context<Self>;
}

#[derive(Debug, Copy, Clone)]
pub struct LevelById(pub u64);

impl Message for LevelById {
    type Result = Option<CacheEntry<Level<Option<NewgroundsSong>, Option<Creator>>, Entry>>;
}

impl Handler<LevelById> for HttpActor {
    type Result = Option<CacheEntry<Level<Option<NewgroundsSong>, Option<Creator>>, Entry>>;

    fn handle(&mut self, LevelById(id): LevelById, ctx: &mut Self::Context) -> Self::Result {
        let future = self
            .gdcf
            .level(id)
            .map_err(|err| error!("GDCF database access failed: {:?}", err))
            .ok()?
            .upgrade::<Level<Option<NewgroundsSong>, _>>()
            .upgrade::<Level<_, Option<Creator>>>();

        let cached = future.clone_cached();

        ctx.spawn(
            future
                .map(|_| ())
                .map_err(|error| error!("Error while refreshing GDCF cache: {}", error))
                .into_actor(self),
        );

        cached.ok()
    }
}

#[derive(Debug)]
pub struct GetDemon(pub String);

impl Message for GetDemon {
    type Result = Option<CacheEntry<Level<Option<u64>, Option<Creator>>, Entry>>;
}

impl Handler<GetDemon> for HttpActor {
    type Result = Option<CacheEntry<Level<Option<u64>, Option<Creator>>, Entry>>;

    fn handle(&mut self, msg: GetDemon, ctx: &mut Context<Self>) -> Self::Result {
        let future = self
            .gdcf
            .levels(
                LevelsRequest::default()
                    .request_type(LevelRequestType::MostLiked)
                    .search(msg.0.clone())
                    .with_rating(LevelRating::Demon(DemonRating::Hard))
                    .filter(SearchFilters::default().rated()),
            )
            .map_err(|err| error!("GDCF database access failed: {:?}", err))
            .ok()?
            .upgrade_all::<PartialLevel<_, Option<Creator>>>()
            .upgrade_all::<Level<_, _>>();

        let cached_clone = future.clone_cached();

        ctx.spawn(
            future
                .map(|_| info!("LevelsRequest successful"))
                .map_err(|err| error!("Error during GDCF cache refresh! {:?}", err))
                .into_actor(self),
        );

        match cached_clone.ok()? {
            CacheEntry::Missing => Some(CacheEntry::Missing),
            CacheEntry::DeducedAbsent => Some(CacheEntry::DeducedAbsent),
            CacheEntry::MarkedAbsent(meta) => Some(CacheEntry::MarkedAbsent(meta)),
            CacheEntry::Cached(demons, meta) =>
                demons
                    .into_iter()
                    .filter(|demon| demon.base.name == msg.0)
                    .max_by(|x, y| x.base.difficulty.cmp(&y.base.difficulty))
                    .map(|demon| CacheEntry::Cached(demon, meta)),
        }
    }
}
