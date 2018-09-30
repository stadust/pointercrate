use actix::{Actor, Addr, Handler, Message, SyncArbiter, SyncContext};
use crate::{
    config::{EXTENDED_LIST_SIZE, LIST_SIZE},
    error::PointercrateError,
    middleware::auth::{Authorization, Claims},
    model::{
        record::{RecordStatus, Submission},
        user::{PatchMe, Registration},
        Demon, Player, Record, Submitter, User,
    },
    patch::{Patchable, UpdateDatabase},
    video,
};
use diesel::{
    pg::PgConnection,
    r2d2::{ConnectionManager, Pool},
    result::Error,
    RunQueryDsl,
};
use ipnetwork::IpNetwork;
use log::{debug, info};

pub struct DatabaseActor(pub Pool<ConnectionManager<PgConnection>>);

impl DatabaseActor {
    pub fn from_env() -> Addr<Self> {
        info!("Initializing pointercrate database connection pool");

        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL is not set");
        let manager = ConnectionManager::<PgConnection>::new(database_url);
        let pool = Pool::builder()
            .build(manager)
            .expect("Failed to create database connection pool");

        SyncArbiter::start(4, move || DatabaseActor(pool.clone()))
    }
}

impl Actor for DatabaseActor {
    type Context = SyncContext<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        info!("Started pointercrate database actor! We can now interact with the database!")
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        info!(
            "Stopped pointercrate database actor! We can no longer interact with the database! :("
        )
    }
}

pub struct SubmitterByIp(pub IpNetwork);

pub struct PlayerByName(pub String);

pub struct DemonByName(pub String);

pub struct ResolveSubmissionData(pub String, pub String);
pub struct ProcessSubmission(pub Submission, pub Submitter);
pub struct RecordById(pub i32);
pub struct DeleteRecordById(pub i32);

pub struct Register(pub Registration);
pub struct UserById(pub i32);
pub struct UserByName(pub String);
pub struct DeleteUserById(pub i32);

pub struct TokenAuth(pub Authorization);
pub struct BasicAuth(pub Authorization);

pub struct Patch<Target, Patch>(pub User, pub Target, pub Patch)
where
    Target: Patchable<Patch> + UpdateDatabase;

// We cannot use the above struct for this, because both 'User' and 'Target' need to be the same
// object, something the ownership system obviously doesn't allow. The alternative would be cloning
// the user once but that's just ugly, so we have this dedicated struct!
pub struct PatchCurrentUser(pub User, pub PatchMe);

impl Message for SubmitterByIp {
    type Result = Result<Submitter, PointercrateError>;
}

impl Handler<SubmitterByIp> for DatabaseActor {
    type Result = Result<Submitter, PointercrateError>;

    fn handle(&mut self, msg: SubmitterByIp, _ctx: &mut Self::Context) -> Self::Result {
        debug!(
            "Attempt to retrieve submitter with IP '{}', creating if not exists!",
            msg.0
        );

        let connection = &*self
            .0
            .get()
            .map_err(|_| PointercrateError::DatabaseConnectionError)?;

        match Submitter::by_ip(&msg.0).first(connection) {
            Ok(submitter) => Ok(submitter),
            Err(Error::NotFound) =>
                Submitter::insert(connection, &msg.0).map_err(PointercrateError::database),
            Err(err) => Err(PointercrateError::database(err)),
        }
    }
}

impl Message for PlayerByName {
    type Result = Result<Player, PointercrateError>;
}

impl Handler<PlayerByName> for DatabaseActor {
    type Result = Result<Player, PointercrateError>;

    fn handle(&mut self, msg: PlayerByName, _ctx: &mut Self::Context) -> Self::Result {
        debug!(
            "Attempt to retrieve player with name '{}', creating if not exists!",
            msg.0
        );

        let connection = &*self
            .0
            .get()
            .map_err(|_| PointercrateError::DatabaseConnectionError)?;

        match Player::by_name(&msg.0).first(connection) {
            Ok(player) => Ok(player),
            Err(Error::NotFound) =>
                Player::insert(connection, &msg.0).map_err(PointercrateError::database),
            Err(err) => Err(PointercrateError::database(err)),
        }
    }
}

impl Message for DemonByName {
    type Result = Result<Demon, PointercrateError>;
}

impl Handler<DemonByName> for DatabaseActor {
    type Result = Result<Demon, PointercrateError>;

    fn handle(&mut self, msg: DemonByName, _ctx: &mut Self::Context) -> Self::Result {
        debug!("Attempting to retrieve demon with name '{}'!", msg.0);

        let connection = &*self
            .0
            .get()
            .map_err(|_| PointercrateError::DatabaseConnectionError)?;

        match Demon::by_name(&msg.0).first(connection) {
            Ok(demon) => Ok(demon),
            Err(Error::NotFound) =>
                Err(PointercrateError::ModelNotFound {
                    model: "Demon",
                    identified_by: msg.0,
                }),
            Err(err) => Err(PointercrateError::database(err)),
        }
    }
}

impl Message for ResolveSubmissionData {
    type Result = Result<(Player, Demon), PointercrateError>;
}

impl Handler<ResolveSubmissionData> for DatabaseActor {
    type Result = Result<(Player, Demon), PointercrateError>;

    fn handle(&mut self, msg: ResolveSubmissionData, ctx: &mut Self::Context) -> Self::Result {
        debug!(
            "Attempt to resolve player '{}' and demon '{}' for a submission!",
            msg.0, msg.1
        );

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
        debug!("Processing submission {:?}", msg.0);

        if msg.1.banned {
            return Err(PointercrateError::BannedFromSubmissions)?
        }

        let Submission {
            progress,
            player,
            demon,
            video,
            verify_only,
        } = msg.0;

        let video = match video {
            Some(ref video) => Some(video::validate(video)?),
            None => None,
        };

        let (player, demon) = self.handle(ResolveSubmissionData(player, demon), ctx)?;

        if player.banned {
            return Err(PointercrateError::PlayerBanned)
        }

        if demon.position > *EXTENDED_LIST_SIZE {
            return Err(PointercrateError::SubmitLegacy)
        }

        if demon.position > *LIST_SIZE && progress != 100 {
            return Err(PointercrateError::Non100Extended)
        }

        if progress > 100 || progress < demon.requirement {
            return Err(PointercrateError::InvalidProgress {
                requirement: demon.requirement,
            })?
        }

        debug!("Submission is valid, checking for duplicates!");

        let connection = &*self
            .0
            .get()
            .map_err(|_| PointercrateError::DatabaseConnectionError)?;

        let record: Result<Record, _> = match video {
            Some(ref video) =>
                Record::get_existing(player.id, &demon.name, video).first(connection),
            None => Record::by_player_and_demon(player.id, &demon.name).first(connection),
        };

        let video_ref = video.as_ref().map(AsRef::as_ref);

        let id = match record {
            Ok(record) =>
                if record.status() != RecordStatus::Rejected && record.progress() < progress {
                    if verify_only {
                        return Ok(None)
                    }

                    if record.status() == RecordStatus::Submitted {
                        debug!(
                            "The submission is duplicated, but new one has higher progress. Deleting old with id {}!",
                            record.id
                        );

                        record
                            .delete(connection)
                            .map_err(PointercrateError::database)?;
                    }

                    debug!(
                        "Duplicate {} either already accepted, or has lower progress, accepting!",
                        record.id
                    );

                    Record::insert(
                        connection,
                        progress,
                        video_ref,
                        player.id,
                        msg.1.id,
                        &demon.name,
                    ).map_err(PointercrateError::database)?
                } else {
                    return Err(PointercrateError::SubmissionExists {
                        status: record.status(),
                    })
                },
            Err(Error::NotFound) => {
                debug!("No duplicate found, accepting!");

                if verify_only {
                    return Ok(None)
                }

                Record::insert(
                    connection,
                    progress,
                    video_ref,
                    player.id,
                    msg.1.id,
                    &demon.name,
                ).map_err(PointercrateError::database)?
            },
            Err(err) => return Err(PointercrateError::database(err)),
        };

        info!("Submission successful! Created new record with ID {}", id);

        Ok(Some(Record {
            id,
            progress,
            video,
            status: RecordStatus::Submitted,
            player,
            submitter: msg.1.id,
            demon: demon.into(),
        }))
    }
}

impl Message for RecordById {
    type Result = Result<Record, PointercrateError>;
}

impl Handler<RecordById> for DatabaseActor {
    type Result = Result<Record, PointercrateError>;

    fn handle(&mut self, msg: RecordById, _: &mut Self::Context) -> Self::Result {
        debug!("Attempt to resolve record by id {}", msg.0);

        let connection = &*self
            .0
            .get()
            .map_err(|_| PointercrateError::DatabaseConnectionError)?;

        match Record::by_id(msg.0).first(connection) {
            Ok(record) => Ok(record),
            Err(Error::NotFound) =>
                Err(PointercrateError::ModelNotFound {
                    model: "Record",
                    identified_by: msg.0.to_string(),
                }),
            Err(err) => Err(PointercrateError::database(err)),
        }
    }
}

impl Message for DeleteRecordById {
    type Result = Result<(), PointercrateError>;
}

impl Handler<DeleteRecordById> for DatabaseActor {
    type Result = Result<(), PointercrateError>;

    fn handle(&mut self, msg: DeleteRecordById, _: &mut Self::Context) -> Self::Result {
        info!("Deleting record with ID {}!", msg.0);

        self.0
            .get()
            .map_err(|_| PointercrateError::DatabaseConnectionError)
            .and_then(|connection| {
                Record::delete_by_id(&connection, msg.0).map_err(PointercrateError::database)
            })
    }
}

impl Message for UserById {
    type Result = Result<User, PointercrateError>;
}

impl Handler<UserById> for DatabaseActor {
    type Result = Result<User, PointercrateError>;

    fn handle(&mut self, msg: UserById, _: &mut Self::Context) -> Self::Result {
        debug!("Attempt to resolve user by id {}", msg.0);

        let connection = &*self
            .0
            .get()
            .map_err(|_| PointercrateError::DatabaseConnectionError)?;

        match User::by_id(msg.0).first(connection) {
            Ok(user) => Ok(user),
            Err(Error::NotFound) =>
                Err(PointercrateError::ModelNotFound {
                    model: "User",
                    identified_by: msg.0.to_string(),
                }),
            Err(err) => Err(PointercrateError::database(err)),
        }
    }
}

impl Message for UserByName {
    type Result = Result<User, PointercrateError>;
}

impl Handler<UserByName> for DatabaseActor {
    type Result = Result<User, PointercrateError>;

    fn handle(&mut self, msg: UserByName, _: &mut Self::Context) -> Self::Result {
        debug!("Attempt to resolve user by name {}", msg.0);

        let connection = &*self
            .0
            .get()
            .map_err(|_| PointercrateError::DatabaseConnectionError)?;

        match User::by_name(&msg.0).first(connection) {
            Ok(user) => Ok(user),
            Err(Error::NotFound) =>
                Err(PointercrateError::ModelNotFound {
                    model: "User",
                    identified_by: msg.0,
                }),
            Err(err) => Err(PointercrateError::database(err)),
        }
    }
}

impl Message for TokenAuth {
    type Result = Result<User, PointercrateError>;
}

// During authorization, all and every error that might come up will be converted into
// `PointercrateError::Unauthorized`
impl Handler<TokenAuth> for DatabaseActor {
    type Result = Result<User, PointercrateError>;

    fn handle(&mut self, msg: TokenAuth, ctx: &mut Self::Context) -> Self::Result {
        debug!("Attempting to perform token authorization (we're not logging the token for obvious reasons smh)");

        if let Authorization::Token(token) = msg.0 {
            // Well this is reassuring. Also we directly deconstruct it and only save the ID so we
            // don't accidentally use unsafe values later on
            let Claims { id, .. } = jsonwebtoken::dangerous_unsafe_decode::<Claims>(&token)
                .map_err(|_| PointercrateError::Unauthorized)?
                .claims;

            debug!("The token identified the user with id {}", id);

            let user = self
                .handle(UserById(id), ctx)
                .map_err(|_| PointercrateError::Unauthorized)?;

            user.validate_token(&token)
        } else {
            Err(PointercrateError::Unauthorized)
        }
    }
}

impl Message for BasicAuth {
    type Result = Result<User, PointercrateError>;
}

impl Handler<BasicAuth> for DatabaseActor {
    type Result = Result<User, PointercrateError>;

    fn handle(&mut self, msg: BasicAuth, ctx: &mut Self::Context) -> Self::Result {
        debug!("Attempting to perform basic authorization (we're not logging the password for even more obvious reasons smh)");

        if let Authorization::Basic(username, password) = msg.0 {
            debug!(
                "Trying to authorize user {} (still not logging the password)",
                username
            );

            let user = self
                .handle(UserByName(username), ctx)
                .map_err(|_| PointercrateError::Unauthorized)?;

            user.verify_password(&password)
        } else {
            Err(PointercrateError::Unauthorized)
        }
    }
}

impl Message for Register {
    type Result = Result<User, PointercrateError>;
}

impl Handler<Register> for DatabaseActor {
    type Result = Result<User, PointercrateError>;

    fn handle(&mut self, msg: Register, _: &mut Self::Context) -> Self::Result {
        let connection = &*self
            .0
            .get()
            .map_err(|_| PointercrateError::DatabaseConnectionError)?;

        // TODO: username and password validation (long enough, etc)

        match User::by_name(&msg.0.name).first::<User>(connection) {
            Ok(_) => Err(PointercrateError::NameTaken),
            Err(Error::NotFound) =>
                User::register(connection, &msg.0).map_err(PointercrateError::database),
            Err(err) => Err(PointercrateError::database(err)),
        }
    }
}

impl Message for DeleteUserById {
    type Result = Result<(), PointercrateError>;
}

impl Handler<DeleteUserById> for DatabaseActor {
    type Result = Result<(), PointercrateError>;

    fn handle(&mut self, msg: DeleteUserById, _: &mut Self::Context) -> Self::Result {
        info!("Deleting user with ID {}!", msg.0);

        self.0
            .get()
            .map_err(|_| PointercrateError::DatabaseConnectionError)
            .and_then(|connection| {
                User::delete_by_id(&connection, msg.0).map_err(PointercrateError::database)
            })
    }
}

impl<T, P> Message for Patch<T, P>
where
    T: Patchable<P> + UpdateDatabase + 'static,
{
    type Result = Result<T, PointercrateError>;
}

impl<T, P> Handler<Patch<T, P>> for DatabaseActor
where
    T: Patchable<P> + UpdateDatabase + 'static,
{
    type Result = Result<T, PointercrateError>;

    fn handle(&mut self, mut msg: Patch<T, P>, _: &mut Self::Context) -> Self::Result {
        let required = msg.1.required_permissions();

        if msg.0.permissions() & required != required {
            return Err(PointercrateError::MissingPermissions { required })
        }

        // Modify the object we're currently working with to validate the values
        msg.1.apply_patch(msg.2)?;

        let connection = &*self
            .0
            .get()
            .map_err(|_| PointercrateError::DatabaseConnectionError)?;

        // Store the modified object in the database
        msg.1
            .update(connection)
            .map_err(PointercrateError::database)
    }
}

impl Message for PatchCurrentUser {
    type Result = Result<User, PointercrateError>;
}

impl Handler<PatchCurrentUser> for DatabaseActor {
    type Result = Result<User, PointercrateError>;

    fn handle(&mut self, mut msg: PatchCurrentUser, _: &mut Self::Context) -> Self::Result {
        msg.0.apply_patch(msg.1)?;

        let connection = &*self
            .0
            .get()
            .map_err(|_| PointercrateError::DatabaseConnectionError)?;

        msg.0
            .update(connection)
            .map_err(PointercrateError::database)
    }
}
