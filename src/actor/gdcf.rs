use actix::{fut::WrapFuture, Actor, Addr, AsyncContext, Context, Handler, Message};
use gdcf::{
    api::request::level::{LevelsRequest, SearchFilters},
    chrono::Duration,
    model::{
        level::{DemonRating, LevelRating},
        Creator, PartialLevel,
    },
    Gdcf, GdcfFuture,
};
use gdcf_dbcache::cache::{DatabaseCache, DatabaseCacheConfig, Pg};
use gdrs::BoomlingsClient;
use log::{error, info};
use tokio::{self, prelude::future::Future};

#[derive(Debug)]
pub struct GdcfActor(Gdcf<BoomlingsClient, DatabaseCache<Pg>>);

impl GdcfActor {
    pub fn from_env() -> Addr<Self> {
        info!("Initalizing GDCF from environment data");

        let gdcf_url = std::env::var("GDCF_DATABASE_URL").expect("GDCF_DATABASE_URL is not set");

        let mut config = DatabaseCacheConfig::postgres_config(&gdcf_url);
        config.invalidate_after(Duration::minutes(30));

        let cache = DatabaseCache::new(config);
        let client = BoomlingsClient::new();

        let actor = GdcfActor::new(client, cache);

        actor.start()
    }
}

impl GdcfActor {
    pub fn new(client: BoomlingsClient, cache: DatabaseCache<Pg>) -> GdcfActor {
        GdcfActor(Gdcf::new(client, cache))
    }
}

impl Actor for GdcfActor {
    type Context = Context<Self>;
}

#[derive(Debug)]
pub struct GetDemon(pub String);

impl Message for GetDemon {
    type Result = Option<PartialLevel<u64, Creator>>;
}

impl Handler<GetDemon> for GdcfActor {
    type Result = Option<PartialLevel<u64, Creator>>;

    fn handle(
        &mut self, msg: GetDemon, ctx: &mut Context<Self>,
    ) -> Option<PartialLevel<u64, Creator>> {
        let GdcfFuture { cached, inner } = self.0.levels::<u64, Creator>(
            LevelsRequest::default()
                .search(msg.0)
                .with_rating(LevelRating::Demon(DemonRating::Hard))
                .filter(SearchFilters::default().rated()),
        );

        if let Some(inner) = inner {
            ctx.spawn(
                inner
                    .map(|_| ())
                    .map_err(|err| error!("Error during GDCF cache refresh! {:?}", err))
                    .into_actor(self),
            );
        }

        match cached {
            Some(inner) => {
                let mut inner = inner.extract();

                if !inner.is_empty() {
                    Some(inner.remove(0))
                } else {
                    None
                }
            },
            None => None,
        }
    }
}
