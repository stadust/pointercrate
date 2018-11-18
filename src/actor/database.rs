use actix::{Actor, Addr, Handler, Message, SyncArbiter, SyncContext};
use crate::{
    error::PointercrateError,
    middleware::{
        auth::{Authorization, Claims},
        cond::IfMatch,
    },
    model::{user::PatchMe, Delete, Get, Hotfix, Patch, Post, User},
    pagination::Paginatable,
    patch::PatchField,
    Result,
};
use diesel::{
    pg::PgConnection,
    r2d2::{ConnectionManager, Pool, PooledConnection},
    Connection, RunQueryDsl,
};
use log::{debug, info};
use std::{hash::Hash, marker::PhantomData};

/// Actor that executes database related actions on a thread pool
#[allow(missing_debug_implementations)]
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

    fn connection(&self) -> Result<PooledConnection<ConnectionManager<PgConnection>>> {
        self.0
            .get()
            .map_err(|_| PointercrateError::DatabaseConnectionError)
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

/// Message that indicates the [`DatabaseActor`] to delete the [`User`] object with the given id.
///
/// ## Errors
/// + [`PointercrateError::ModelNotFound`]: Should no user with the given id exist.
//#[derive(Debug)]
//pub struct DeleteUserById(pub i32, pub IfMatch);

/// Message that indicates the [`DatabaseActor`] to authorize a [`User`] by access token
///
/// ## Errors
/// + [`PointercrateError::Unauthorized`]: Authorization failed
#[derive(Debug)]
pub struct TokenAuth(pub Authorization);

/// Message that indicates the [`DatabaseActor`] to authorize a [`User`] using basic auth
///
/// ## Errors
/// + [`PointercrateError::Unauthorized`]: Authorization failed
#[derive(Debug)]
pub struct BasicAuth(pub Authorization);

/// Message that indicates the [`DatabaseActor`] to invalidate all access tokens to the account
/// authorized by the given [`Authorization`] object. The [`Authorization`] object must be of type
/// [`Authorization::Basic] for this.
///
/// Invalidation is done by re-randomizing the salt used for hashing the user's password (since the
/// key tokens are signed with contains the salt, this will invalidate all old access tokens).
///
/// ## Errors
/// + [`PointercrateError::Unauthorized`]: Authorization failed
#[derive(Debug)]
pub struct Invalidate(pub Authorization);

#[derive(Debug)]
pub struct Paginate<P: Paginatable>(pub P);

impl Message for TokenAuth {
    type Result = Result<User>;
}

// During authorization, all and every error that might come up will be converted into
// `PointercrateError::Unauthorized`
impl Handler<TokenAuth> for DatabaseActor {
    type Result = Result<User>;

    fn handle(&mut self, msg: TokenAuth, _: &mut Self::Context) -> Self::Result {
        debug!("Attempting to perform token authorization (we're not logging the token for obvious reasons smh)");

        if let Authorization::Token(token) = msg.0 {
            // Well this is reassuring. Also we directly deconstruct it and only save the ID so we
            // don't accidentally use unsafe values later on
            let Claims { id, .. } = jsonwebtoken::dangerous_unsafe_decode::<Claims>(&token)
                .map_err(|_| PointercrateError::Unauthorized)?
                .claims;

            debug!("The token identified the user with id {}", id);

            let user =
                User::get(id, &*self.connection()?).map_err(|_| PointercrateError::Unauthorized)?;

            user.validate_token(&token)
        } else {
            Err(PointercrateError::Unauthorized)
        }
    }
}

impl Message for Invalidate {
    type Result = Result<()>;
}

impl Handler<Invalidate> for DatabaseActor {
    type Result = Result<()>;

    fn handle(&mut self, msg: Invalidate, ctx: &mut Self::Context) -> Self::Result {
        if let Authorization::Basic(_, ref password) = msg.0 {
            let password = password.clone();
            let user = self.handle(BasicAuth(msg.0), ctx)?;
            let patch = PatchMe {
                password: PatchField::Some(password),
                display_name: PatchField::Absent,
                youtube_channel: PatchField::Absent,
            };

            self.handle(
                PatchMessage::<_, User, _>::unconditional(user.id, patch),
                ctx,
            )
            .map(|_| ())
        } else {
            Err(PointercrateError::Unauthorized)
        }
    }
}

impl Message for BasicAuth {
    type Result = Result<User>;
}

impl Handler<BasicAuth> for DatabaseActor {
    type Result = Result<User>;

    fn handle(&mut self, msg: BasicAuth, _: &mut Self::Context) -> Self::Result {
        debug!("Attempting to perform basic authorization (we're not logging the password for even more obvious reasons smh)");

        if let Authorization::Basic(username, password) = msg.0 {
            debug!(
                "Trying to authorize user {} (still not logging the password)",
                username
            );

            let user = User::get(username, &*self.connection()?)
                .map_err(|_| PointercrateError::Unauthorized)?;

            user.verify_password(&password)
        } else {
            Err(PointercrateError::Unauthorized)
        }
    }
}

impl<P: Paginatable + 'static> Message for Paginate<P> {
    type Result = Result<(Vec<P::Result>, String)>;
}

impl<P: Paginatable + 'static> Handler<Paginate<P>> for DatabaseActor {
    type Result = Result<(Vec<P::Result>, String)>;

    fn handle(&mut self, msg: Paginate<P>, _: &mut Self::Context) -> Self::Result {
        let connection = &*self
            .0
            .get()
            .map_err(|_| PointercrateError::DatabaseConnectionError)?;

        let first = msg.0.first(connection)?;
        let last = msg.0.last(connection)?;
        let next = msg.0.next_after(connection)?;
        let prev = msg.0.prev_before(connection)?;

        let result = msg.0.result(connection)?;

        // TODO: compare last thing in our list with last and first thing in our list with first
        // and then only generate the needed headers

        let header = format! {
            "<{}>; rel=first,<{}>; rel=prev,<{}>; rel=next,<{}>; rel=last",
            serde_urlencoded::ser::to_string(first).unwrap(),
            serde_urlencoded::ser::to_string(prev).unwrap(),
            serde_urlencoded::ser::to_string(next).unwrap(),
            serde_urlencoded::ser::to_string(last).unwrap(),
        };

        Ok((result, header))
    }
}

#[derive(Debug)]
pub struct GetMessage<Key, G: Get<Key>>(pub Key, pub PhantomData<G>);

impl<Key, G: Get<Key> + 'static> Message for GetMessage<Key, G> {
    type Result = Result<G>;
}

impl<Key, G: Get<Key> + 'static> Handler<GetMessage<Key, G>> for DatabaseActor {
    type Result = Result<G>;

    fn handle(&mut self, msg: GetMessage<Key, G>, _: &mut Self::Context) -> Self::Result {
        G::get(msg.0, &*self.connection()?)
    }
}

#[derive(Debug)]
pub struct PostMessage<T, P: Post<T> + 'static>(pub T, pub PhantomData<P>);

impl<T, P: Post<T> + 'static> Message for PostMessage<T, P> {
    type Result = Result<P>;
}

impl<T, P: Post<T> + 'static> Handler<PostMessage<T, P>> for DatabaseActor {
    type Result = Result<P>;

    fn handle(&mut self, msg: PostMessage<T, P>, _: &mut Self::Context) -> Self::Result {
        P::create_from(msg.0, &*self.connection()?)
    }
}

#[derive(Debug)]
pub struct DeleteMessage<Key, D>(pub Key, pub Option<IfMatch>, pub PhantomData<D>)
where
    D: Get<Key> + Delete + Hash;

impl<Key, D> DeleteMessage<Key, D>
where
    D: Get<Key> + Delete + Hash,
{
    pub fn unconditional(key: Key) -> Self {
        DeleteMessage(key, None, PhantomData)
    }
}

impl<Key, D> Message for DeleteMessage<Key, D>
where
    D: Get<Key> + Delete + Hash,
{
    type Result = Result<()>;
}

impl<Key, D> Handler<DeleteMessage<Key, D>> for DatabaseActor
where
    D: Get<Key> + Delete + Hash,
{
    type Result = Result<()>;

    fn handle(&mut self, msg: DeleteMessage<Key, D>, _: &mut Self::Context) -> Self::Result {
        let connection = &*self.connection()?;

        connection.transaction(|| {
            let target = D::get(msg.0, connection)?;

            match msg.1 {
                Some(condition) => target.delete_if_match(condition, connection),
                None => target.delete(connection),
            }
        })
    }
}

#[derive(Debug)]
pub struct PatchMessage<Key, P, H>(pub Key, pub H, pub Option<IfMatch>, pub PhantomData<P>)
where
    H: Hotfix,
    P: Get<Key> + Patch<H> + Hash;

impl<Key, P, H> PatchMessage<Key, P, H>
where
    H: Hotfix,
    P: Get<Key> + Patch<H> + Hash,
{
    pub fn unconditional(key: Key, fix: H) -> Self {
        PatchMessage(key, fix, None, PhantomData)
    }
}

impl<Key, P, H> Message for PatchMessage<Key, P, H>
where
    H: Hotfix,
    P: Get<Key> + Patch<H> + Hash + 'static,
{
    type Result = Result<P>;
}

impl<Key, P, H> Handler<PatchMessage<Key, P, H>> for DatabaseActor
where
    H: Hotfix,
    P: Get<Key> + Patch<H> + Hash + 'static,
{
    type Result = Result<P>;

    fn handle(&mut self, msg: PatchMessage<Key, P, H>, _: &mut Self::Context) -> Self::Result {
        let connection = &*self.connection()?;

        connection.transaction(|| {
            let target = P::get(msg.0, connection)?;

            match msg.2 {
                Some(condition) => target.patch_if_match(msg.1, condition, connection),
                None => target.patch(msg.1, connection),
            }
        })
    }
}
