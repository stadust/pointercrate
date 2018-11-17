#[macro_use]
pub mod user;
pub mod audit;
pub mod demon;
pub mod player;
pub mod record;
pub mod submitter;

pub use self::{
    demon::Demon,
    player::Player,
    record::Record,
    submitter::Submitter,
    user::{Permissions, User},
};
use crate::{error::PointercrateError, middleware::cond::IfMatch, Result};
use diesel::{pg::PgConnection, query_dsl::methods::SelectDsl, Expression, QuerySource};
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

/// Trait that marks its implementors as models
pub trait Model {
    /// The database column this [`Model`] maps to
    type Columns: Expression;

    /// The database table this [`Model`] maps to
    type Table: SelectDsl<Self::Columns> + QuerySource;

    /// Constructs a simple diesel `SELECT`-query that returns all columns of rows of this
    /// [`Model`]'s from the database
    fn all() -> diesel::dsl::Select<Self::Table, Self::Columns>;
}

pub trait Get<Key>: Sized {
    fn get(id: Key, connection: &PgConnection) -> Result<Self>;
}

pub trait Post<T>: Sized {
    fn create_from(from: T, connection: &PgConnection) -> Result<Self>;
}

/// Trait marking its implementors as containing patch data which can be applied to a matching
/// [`Patchable`]
///
/// TODO: find  better name
pub trait Hotfix {
    /// The level of authorization required to perform this [`Patch`]
    ///
    /// The default implementation allows all patches without any authorization
    fn required_permissions(&self) -> Permissions {
        Permissions::empty()
    }
}

pub trait Patch<P: Hotfix>: Sized {
    fn patch(self, patch: P, connection: &PgConnection) -> Result<Self>;

    fn patch_if_match(self, patch: P, condition: IfMatch, connection: &PgConnection) -> Result<Self>
    where
        Self: Hash,
    {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);

        if condition.met(hasher.finish()) {
            self.patch(patch, connection)
        } else {
            Err(PointercrateError::PreconditionFailed)
        }
    }
}

pub trait Delete {
    fn delete(self, connection: &PgConnection) -> Result<()>;

    fn delete_if_match(self, condition: IfMatch, connection: &PgConnection) -> Result<()>
    where
        Self: Hash + Sized,
    {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);

        if condition.met(hasher.finish()) {
            self.delete(connection)
        } else {
            Err(PointercrateError::PreconditionFailed)
        }
    }
}
