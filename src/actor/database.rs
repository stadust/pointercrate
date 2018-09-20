use actix::{Actor, Context, Handler, Message, SyncContext};
use api::record::Submission;
use diesel::{
    pg::PgConnection,
    r2d2::{ConnectionManager, Pool},
    result::Error,
    RunQueryDsl,
};
use error::PointercrateError;
use ipnetwork::IpNetwork;
use model::{Demon, Player, Submitter};

pub struct DatabaseActor(pub Pool<ConnectionManager<PgConnection>>);

impl Actor for DatabaseActor {
    type Context = SyncContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        info!("Started pointercrate database actor! We can now interact with the database!")
    }

    fn stopped(&mut self, ctx: &mut Self::Context) {
        info!("Stopped pointercrate database actor! We ca no longer interact with the database! :(")
    }
}

pub struct SubmitterByIp(pub IpNetwork);
pub struct PlayerByName(pub String);
pub struct DemonByName(pub String);
pub struct ResolveSubmission(pub Submission);

impl Message for SubmitterByIp {
    type Result = Result<Submitter, PointercrateError>;
}

impl Handler<SubmitterByIp> for DatabaseActor {
    type Result = Result<Submitter, PointercrateError>;

    fn handle(&mut self, msg: SubmitterByIp, _ctx: &mut Self::Context) -> Self::Result {
        let connection = &*self.0.get().map_err(|_| PointercrateError::DatabaseConnectionError)?;

        match Submitter::by_ip(&msg.0).first(connection) {
            Ok(submitter) => Ok(submitter),
            Err(Error::NotFound) => Submitter::insert(connection, &msg.0).map_err(|_| PointercrateError::DatabaseError),
            Err(_) => Err(PointercrateError::DatabaseError),
        }
    }
}

impl Message for PlayerByName {
    type Result = Result<Player, PointercrateError>;
}

impl Handler<PlayerByName> for DatabaseActor {
    type Result = Result<Player, PointercrateError>;

    fn handle(&mut self, msg: PlayerByName, ctx: &mut Self::Context) -> Self::Result {
        let connection = &*self.0.get().map_err(|_| PointercrateError::DatabaseConnectionError)?;

        match Player::by_name(&msg.0).first(connection) {
            Ok(player) => Ok(player),
            Err(Error::NotFound) => Player::insert(connection, &msg.0).map_err(|_| PointercrateError::DatabaseError),
            Err(_) => Err(PointercrateError::DatabaseError),
        }
    }
}

impl Message for DemonByName {
    type Result = Result<Demon, PointercrateError>;
}

impl Handler<DemonByName> for DatabaseActor {
    type Result = Result<Demon, PointercrateError>;

    fn handle(&mut self, msg: DemonByName, ctx: &mut Self::Context) -> Self::Result {
        let connection = &*self.0.get().map_err(|_| PointercrateError::DatabaseConnectionError)?;

        match Demon::by_name(&msg.0).first(connection) {
            Ok(demon) => Ok(demon),
            Err(Error::NotFound) =>
                Err(PointercrateError::ModelNotFound {
                    model: "Demon",
                    identified_by: msg.0,
                }),
            Err(_) => Err(PointercrateError::DatabaseError),
        }
    }
}

impl Message for ResolveSubmission {
    type Result = Result<(i16, Player, Demon, Option<String>, bool), PointercrateError>;
}

impl Handler<ResolveSubmission> for DatabaseActor {
    type Result = Result<(i16, Player, Demon, Option<String>, bool), PointercrateError>;

    fn handle(&mut self, msg: ResolveSubmission, ctx: &mut Self::Context) -> Self::Result {
        let Submission {
            progress,
            player,
            demon,
            video,
            verify_only,
        } = msg.0;

        let player = self.handle(PlayerByName(player), ctx)?;
        let demon = self.handle(DemonByName(demon), ctx)?;

        Ok((progress, player, demon, video, verify_only))
    }
}
