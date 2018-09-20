use actix::{Actor, Context, Handler, Message};
use gdcf::{
    api::request::{LevelsRequest, UserRequest},
    cache::CachedObject,
    model::{PartialLevel, User},
    Gdcf, GdcfFuture,
};
use gdcf_dbcache::cache::{DatabaseCache, Pg};
use gdrs::BoomlingsClient;
use tokio::{self, prelude::future::Future};

pub struct GdcfActor(Gdcf<BoomlingsClient, DatabaseCache<Pg>>);

impl GdcfActor {
    pub fn new(client: BoomlingsClient, cache: DatabaseCache<Pg>) -> GdcfActor {
        GdcfActor(Gdcf::new(client, cache))
    }
}

impl Actor for GdcfActor {
    type Context = Context<Self>;
}

pub struct UserRequestMessage(pub UserRequest);

impl Into<UserRequestMessage> for UserRequest {
    fn into(self) -> UserRequestMessage {
        UserRequestMessage(self)
    }
}
impl Message for UserRequestMessage {
    type Result = Option<User>;
}

impl Handler<UserRequestMessage> for GdcfActor {
    type Result = Option<User>;

    fn handle(&mut self, msg: UserRequestMessage, _ctx: &mut Context<Self>) -> Option<User> {
        let GdcfFuture { cached, inner } = self.0.user(msg.0);

        if let Some(inner) = inner {
            tokio::spawn(
                inner
                    .map(|_| ())
                    .map_err(|err| error!("Error during GDCF cache refresh! {:?}", err)),
            );
        }

        cached.map(CachedObject::extract)
    }
}

pub struct LevelsRequestMessage(pub LevelsRequest);

impl Into<LevelsRequestMessage> for LevelsRequest {
    fn into(self) -> LevelsRequestMessage {
        LevelsRequestMessage(self)
    }
}

impl Message for LevelsRequestMessage {
    type Result = Option<Vec<PartialLevel<u64, u64>>>;
}

impl Handler<LevelsRequestMessage> for GdcfActor {
    type Result = Option<Vec<PartialLevel<u64, u64>>>;

    fn handle(&mut self, msg: LevelsRequestMessage, _ctx: &mut Context<Self>) -> Option<Vec<PartialLevel<u64, u64>>> {
        let GdcfFuture { cached, inner } = self.0.levels(msg.0);

        if let Some(inner) = inner {
            tokio::spawn(
                inner
                    .map(|_| ())
                    .map_err(|err| error!("Error during GDCF cache refresh! {:?}", err)),
            );
        }

        cached.map(CachedObject::extract)
    }
}
