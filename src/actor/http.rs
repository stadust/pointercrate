use crate::{actor::database::DeleteMessage, model::record::Record};
use actix::{fut::WrapFuture, Actor, Addr, AsyncContext, Context, Handler, Message, Recipient};
use gdcf::{
    api::request::level::{LevelRequestType, LevelsRequest, SearchFilters},
    cache::CachedObject,
    chrono::Duration,
    model::{
        level::{DemonRating, Level, LevelRating},
        Creator, NewgroundsSong,
    },
    Gdcf, GdcfFuture,
};
use gdcf_dbcache::cache::{DatabaseCache, DatabaseCacheConfig, Pg};
use gdrs::BoomlingsClient;
use hyper::{
    client::{Client, HttpConnector},
    Body, Request,
};
use hyper_tls::HttpsConnector;
use log::{debug, error, info, warn};
use serde_json::json;
use std::sync::Arc;
use tokio::{
    self,
    prelude::future::{result, Either, Future},
};

/// Actor for whatever the fuck just happens to need to be done and isn't database access
#[allow(missing_debug_implementations)]
pub struct HttpActor {
    gdcf: Gdcf<BoomlingsClient, DatabaseCache<Pg>>,
    http_client: Client<HttpsConnector<HttpConnector>>,
    discord_webhook_url: Arc<Option<String>>,
    deletor: Recipient<DeleteMessage<i32, Record>>,
}

impl HttpActor {
    pub fn from_env(deletor: Recipient<DeleteMessage<i32, Record>>) -> Addr<Self> {
        info!("Initalizing HttpActor from environment data");

        let gdcf_url = std::env::var("GDCF_DATABASE_URL").expect("GDCF_DATABASE_URL is not set");

        let mut config = DatabaseCacheConfig::postgres_config(&gdcf_url)
            .expect("Failed to connect to GDCF database");
        config.invalidate_after(Duration::minutes(30));

        let cache = DatabaseCache::new(config);
        let client = BoomlingsClient::new();

        cache.initialize().unwrap();

        HttpActor {
            deletor,
            gdcf: Gdcf::new(client, cache),
            http_client: Client::builder().build(HttpsConnector::new(4).unwrap()),
            discord_webhook_url: Arc::new(std::env::var("DISCORD_WEBHOOK").ok()),
        }
        .start()
    }

    pub fn execute_discord_webhook(
        &self, data: serde_json::Value,
    ) -> impl Future<Item = (), Error = ()> {
        if let Some(ref uri) = *self.discord_webhook_url {
            info!("Executing discord webhook!");

            let request = Request::post(uri)
                .header("Content-Type", "application/json")
                .body(Body::from(data.to_string()))
                .unwrap();

            let future = self
                .http_client
                .request(request)
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

        let request = Request::head(url).body(Body::empty()).unwrap();

        self.http_client
            .request(request)
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

#[derive(Debug)]
pub struct LevelById(pub u64);

impl Message for LevelById {
    type Result = Option<Level<NewgroundsSong, Creator>>;
}

impl Handler<LevelById> for HttpActor {
    type Result = Option<Level<NewgroundsSong, Creator>>;

    fn handle(&mut self, LevelById(id): LevelById, ctx: &mut Self::Context) -> Self::Result {
        let GdcfFuture { cached, inner } = self.gdcf.level(id.into());

        if let Some(inner) = inner {
            ctx.spawn(inner.map(|_| ()).map_err(|_| ()).into_actor(self));
        }

        cached.map(CachedObject::extract)
    }
}

#[derive(Debug)]
pub struct GetDemon(pub String);

impl Message for GetDemon {
    type Result = Option<Level<u64, Creator>>;
}

impl Handler<GetDemon> for HttpActor {
    type Result = Option<Level<u64, Creator>>;

    fn handle(&mut self, msg: GetDemon, ctx: &mut Context<Self>) -> Option<Level<u64, Creator>> {
        let GdcfFuture { cached, inner } = self.gdcf.levels::<u64, Creator>(
            LevelsRequest::default()
                .request_type(LevelRequestType::MostLiked)
                .search(msg.0.clone())
                .with_rating(LevelRating::Demon(DemonRating::Hard))
                .filter(SearchFilters::default().rated()),
        );

        if let Some(inner) = inner {
            debug!(
                "Spawning future to asynchronously perform LevelsRequest for {}",
                msg.0
            );

            ctx.spawn(
                inner
                    .map(|_| info!("LevelsRequest successful"))
                    .map_err(|err| error!("Error during GDCF cache refresh! {:?}", err))
                    .into_actor(self),
            );
        }

        match cached {
            Some(inner) => {
                let mut inner = inner.extract();

                if !inner.is_empty() {
                    let best_match = inner.iter().max_by(|x, y| x.difficulty.cmp(&y.difficulty)).unwrap();

                    let GdcfFuture { cached, inner } = self.gdcf.level(best_match.level_id.into());

                    if let Some(inner) = inner {
                        debug!(
                            "Spawning future to asynchronously perform LevelRequest for {}",
                            msg.0
                        );

                        ctx.spawn(
                            inner
                                .map(|level| info!("Successfully retrieved level {}", level))
                                .map_err(|err| error!("Error during GDCF cache refresh! {:?}", err))
                                .into_actor(self),
                        );
                    }

                    cached.map(CachedObject::extract)
                } else {
                    None
                }
            },
            None => None,
        }
    }
}

#[derive(Debug)]
pub struct PostProcessRecord(pub Option<Record>);

impl Message for PostProcessRecord {
    type Result = Option<Record>;
}

impl Handler<PostProcessRecord> for HttpActor {
    type Result = Option<Record>;

    fn handle(
        &mut self, PostProcessRecord(record): PostProcessRecord, ctx: &mut Self::Context,
    ) -> Self::Result {
        if let Some(ref record) = record {
            info!("Post processing record {}", record);

            let record_id = record.id;
            let progress = f32::from(record.progress) / 100f32;

            let mut payload = json!({
                "content": format!("**New record submitted! ID: {}**", record_id),
                "embeds": [
                    {
                        "type": "rich",
                        "title": format!("{}% on {}", record.progress, record.demon.name),
                        "description": format!("{} just got {}% on {}! Go add his record!", record.player.name, record.progress, record.demon.name),
                        "footer": {
                            "text": format!("This record has been submitted by submitter #{}", record.submitter.unwrap_or(1))
                        },
                        "color": (0x9e0000 as f32 * progress) as i32 & 0xFF0000 + (0x00e000 as f32 * progress) as i32 & 0x00FF00,
                        "author": {
                            "name": format!("{} (ID: {})", record.player.name, record.player.id),
                            "url": record.video
                        },
                        "thumbnail": {
                            "url": "https://cdn.discordapp.com/avatars/277391246035648512/b03c85d94dc02084c413a7fdbe2cea79.webp?size=1024"
                        },
                    }
                ]
            });

            if let Some(ref video) = record.video {
                // FIXME: this isn't supported by discord. We need to figure out another way then :(
                payload["embeds"][0]["video"] = json! {
                    {"url": video}
                };
                payload["embeds"][0]["fields"] = json! {
                    [{
                        "name": "Video Proof:",
                        "value": video
                    }]
                };
            }

            let deletor = self.deletor.clone();
            let payload_future = self.execute_discord_webhook(payload);

            if let Some(ref video) = record.video {
                debug!("Asynchronously validating video '{}'", video);

                let future = self.if_exists(video).or_else(move |_| {
                    warn!("A HEAD request to video yielded an error response, automatically deleting submission!");

                    deletor
                        .send(DeleteMessage::unconditional(record_id))
                        .map_err(move |error| error!("INTERNAL SERVER ERROR: Failure to delete record {} - {:?}!", record_id, error))
                        .map(|_| ())
                        .and_then(|_| Err(()))
                });

                ctx.spawn(future.and_then(move |_| payload_future).into_actor(self));
            } else {
                ctx.spawn(payload_future.into_actor(self));
            }
        }

        record
    }
}
