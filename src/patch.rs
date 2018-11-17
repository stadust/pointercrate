use crate::{model::user::Permissions, Result};
use diesel::{PgConnection, QueryResult};
use serde::{Deserialize, Deserializer};

pub use crate::model::Hotfix as Patch;

macro_rules! patch {
    ($target: expr, $patch: ident, $field: ident) => {
        match $patch.$field {
            PatchField::Some($field) => $target.$field = Some($field),
            PatchField::Null => $target.$field = None,
            _ => (),
        }
    };

    ($target: expr, $patch: ident, $field: ident, $method: ident) => {
        match $patch.$field {
            PatchField::Some($field) => $target.$method(&$field),
            PatchField::Null => $target.$field = None,
            _ => (),
        }
    };
}

macro_rules! patch_not_null {
    ($target: expr, $patch: ident, $field: ident) => {
        match $patch.$field {
            PatchField::Some($field) => $target.$field = $field,
            PatchField::Null =>
                return Err(PointercrateError::UnexpectedNull {
                    field: stringify!($field),
                }),
            _ => (),
        }
    };

    ($target: expr, $patch: ident, $field: ident, $method: ident) => {
        match $patch.$field {
            PatchField::Some($field) => $target.$method(&$field),
            PatchField::Null =>
                return Err(PointercrateError::UnexpectedNull {
                    field: stringify!($field),
                }),
            _ => (),
        }
    };

    ($target: expr, $patch: ident, $field: ident, *$method: ident) => {
        match $patch.$field {
            PatchField::Some($field) => $target.$method($field),
            PatchField::Null =>
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
                pub $field: PatchField<$t>,
            )*
        }
    }
}

/// Trait that indicates that an object can be patched using some patch-data `T`
pub trait Patchable<T>
where
    T: Patch,
{
    /// Applies the given patch in-place.
    ///
    /// ## Errors
    /// If the patch was semantically invalid, an [`Err`] should be returned.
    fn apply_patch(&mut self, patch: T) -> Result<()>;

    /// Updates the database copy of this [`Patchable`] by updating every field that might have
    /// changed
    fn update_database(&self, connection: &PgConnection) -> Result<()>;
}

#[derive(Debug)]
pub enum PatchField<T> {
    Null,
    Absent,
    Some(T),
}

impl<T> Default for PatchField<T> {
    fn default() -> Self {
        PatchField::Absent
    }
}

pub(crate) fn deserialize_patch<'de, T, D>(
    deserializer: D,
) -> std::result::Result<PatchField<T>, D::Error>
where
    T: Deserialize<'de>,
    D: Deserializer<'de>,
{
    let value: Option<T> = Deserialize::deserialize(deserializer)?;

    match value {
        Some(t) => Ok(PatchField::Some(t)),
        None => Ok(PatchField::Null),
    }
}
