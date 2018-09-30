use crate::{error::PointercrateError, model::user::Permissions};
use diesel::{PgConnection, QueryResult};
use serde::{Deserialize, Deserializer};

macro_rules! patch {
    ($target: expr, $patch: ident, $field: ident) => {
        match $patch.$field {
            Patch::Some($field) => $target.$field = Some($field),
            Patch::Null => $target.$field = None,
            _ => (),
        }
    };

    ($target: expr, $patch: ident, $field: ident, $method: ident) => {
        match $patch.$field {
            Patch::Some($field) => $target.$method(&$field),
            Patch::Null => $target.$field = None,
            _ => (),
        }
    };
}

macro_rules! patch_not_null {
    ($target: expr, $patch: ident, $field: ident) => {
        match $patch.$field {
            Patch::Some($field) => $target.$field = Some($field),
            Patch::Null =>
                return Err(PointercrateError::UnexpectedNull {
                    field: stringify!($field),
                }),
            _ => (),
        }
    };

    ($target: expr, $patch: ident, $field: ident, $method: ident) => {
        match $patch.$field {
            Patch::Some($field) => $target.$method(&$field),
            Patch::Null =>
                return Err(PointercrateError::UnexpectedNull {
                    field: stringify!($field),
                }),
            _ => (),
        }
    };
}

macro_rules! make_patch {
    (struct $name: ident {$($field: ident:$t:ty),*}) => {
        #[derive(Deserialize, Debug)]
        pub struct $name {
            $(
                #[serde(default, deserialize_with = "deserialize_patch")]
                pub $field: Patch<$t>,
            )*
        }
    }
}

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

pub(crate) fn deserialize_patch<'de, T, D>(deserializer: D) -> Result<Patch<T>, D::Error>
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
