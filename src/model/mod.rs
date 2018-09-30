pub mod audit;
pub mod demon;
pub mod player;
pub mod record;
pub mod submitter;
pub mod user;

use self::user::Permissions;
pub use self::{demon::Demon, player::Player, record::Record, submitter::Submitter, user::User};
use crate::error::PointercrateError;
use diesel::{PgConnection, QueryResult};
use serde::{Deserialize, Deserializer};

pub trait Patchable<T> {
    fn apply_patch(&mut self, patch: T) -> Result<(), PointercrateError>;

    fn required_permissions(&self) -> Permissions;
}

pub trait UpdateDatabase: Sized {
    fn update(self, connection: &PgConnection) -> QueryResult<Self>;
}

#[derive(Debug)]
pub enum Patch<T> {
    Null,
    Absent,
    Some(T),
}

impl<T> Default for Patch<T> {
    fn default() -> Self {
        Patch::Absent
    }
}

fn deserialize_patch<'de, T, D>(deserializer: D) -> Result<Patch<T>, D::Error>
where
    T: Deserialize<'de>,
    D: Deserializer<'de>,
{
    let value: Option<T> = Deserialize::deserialize(deserializer)?;

    match value {
        Some(t) => Ok(Patch::Some(t)),
        None => Ok(Patch::Null),
    }
}
