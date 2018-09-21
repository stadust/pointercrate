use actix::{Actor, Handler, Message, SyncContext};
use crate::{
    api::record::Submission,
    error::PointercrateError,
    model::{record::RecordStatus, Demon, Player, Record, Submitter},
};
use diesel::{
    pg::PgConnection,
    r2d2::{ConnectionManager, Pool},
    result::Error,
    RunQueryDsl,
};
use ipnetwork::IpNetwork;
use log::info;

pub const LIST_SIZE: i16 = 50;
pub const EXTENDED_LIST_SIZE: i16 = 100;

pub struct DatabaseActor(pub Pool<ConnectionManager<PgConnection>>);

impl Actor for DatabaseActor {
    type Context = SyncContext<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        info!("Started pointercrate database actor! We can now interact with the database!")
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        info!("Stopped pointercrate database actor! We ca no longer interact with the database! :(")
    }
}

pub struct SubmitterByIp(pub IpNetwork);
pub struct PlayerByName(pub String);
pub struct DemonByName(pub String);
pub struct ResolveSubmissionData(pub String, pub String);
pub struct ProcessSubmission(pub Submission, pub Submitter);

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

    fn handle(&mut self, msg: PlayerByName, _ctx: &mut Self::Context) -> Self::Result {
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

    fn handle(&mut self, msg: DemonByName, _ctx: &mut Self::Context) -> Self::Result {
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

impl Message for ResolveSubmissionData {
    type Result = Result<(Player, Demon), PointercrateError>;
}

impl Handler<ResolveSubmissionData> for DatabaseActor {
    type Result = Result<(Player, Demon), PointercrateError>;

    fn handle(&mut self, msg: ResolveSubmissionData, ctx: &mut Self::Context) -> Self::Result {
        let (player, demon) = (msg.0, msg.1);

        let player = self.handle(PlayerByName(player), ctx)?;
        let demon = self.handle(DemonByName(demon), ctx)?;

        Ok((player, demon))
    }
}

impl Message for ProcessSubmission {
    type Result = Result<Option<Record>, PointercrateError>;
}

impl Handler<ProcessSubmission> for DatabaseActor {
    type Result = Result<Option<Record>, PointercrateError>;

    fn handle(&mut self, msg: ProcessSubmission, ctx: &mut Self::Context) -> Self::Result {
        if msg.1.banned() {
            return Err(PointercrateError::BannedFromSubmissions)?
        }

        let Submission {
            progress,
            player,
            demon,
            video,
            verify_only,
        } = msg.0;

        let (player, demon) = self.handle(ResolveSubmissionData(player, demon), ctx)?;

        if player.banned {
            return Err(PointercrateError::PlayerBanned)
        }

        if demon.position() > EXTENDED_LIST_SIZE {
            return Err(PointercrateError::SubmitLegacy)
        }

        if demon.position() > LIST_SIZE && progress != 100 {
            return Err(PointercrateError::Non100Extended)
        }

        if progress > 100 || progress < demon.requirement() {
            return Err(PointercrateError::InvalidProgress {
                requirement: demon.requirement(),
            })?
        }

        let connection = &*self.0.get().map_err(|_| PointercrateError::DatabaseConnectionError)?;

        let record: Result<Record, _> = match video {
            Some(ref video) => Record::get_existing(player.id, demon.name(), video).first(connection),
            None => Record::by_player_and_demon(player.id, demon.name()).first(connection),
        };

        let id = match record {
            Ok(record) =>
                if record.status() == RecordStatus::Submitted || record.status() == RecordStatus::Approved && record.progress() < progress {
                    if verify_only {
                        return Ok(None)
                    }

                    if record.status() == RecordStatus::Submitted {
                        record.delete(connection).map_err(|_| PointercrateError::DatabaseError)?;
                    }

                    Record::insert(connection, progress, video, player.id, msg.1.id(), demon.name())
                        .map_err(|_| PointercrateError::DatabaseError)?
                } else {
                    return Err(PointercrateError::SubmissionExists { status: record.status() })
                },
            Err(Error::NotFound) => {
                if verify_only {
                    return Ok(None)
                }

                Record::insert(connection, progress, video, player.id, msg.1.id(), demon.name())
                    .map_err(|_| PointercrateError::DatabaseError)?
            },
            Err(_) => return Err(PointercrateError::DatabaseError),
        };

        // TODO: maybe don't re-query the database but instead construct a Record object from the data
        // we already have.
        Record::by_id(id).first(connection)
            .map_err(|_| PointercrateError::DatabaseError)
            .map(Some)
    }
}
