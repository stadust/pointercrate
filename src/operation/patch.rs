// We're gonna allow unused_macros here because they unused onces are here for completeness sake and
// not having them fucks with my OCD. And we might need them one day if we implement crazy weird
// patch operations, who knows
#![allow(unused_macros)]

use crate::{
    error::PointercrateError, middleware::cond::IfMatch, permissions::PermissionsSet, Result,
};
use diesel::pg::PgConnection;
use serde::{de::Error, Deserialize, Deserializer};
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

/// Trait marking its implementors as containing patch data which can be applied to a matching
/// [`Patch`]
pub trait Hotfix {
    /// The level of authorization required to perform this [`HotFix`]
    ///
    /// The default implementation allows all patches without any authorization
    fn required_permissions(&self) -> PermissionsSet;
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

pub fn deserialize_optional<'de, T, D>(
    deserializer: D,
) -> std::result::Result<Option<Option<T>>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Ok(Some(Option::deserialize(deserializer)?))
}

pub fn deserialize_non_optional<'de, T, D>(
    deseralizer: D,
) -> std::result::Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    match Option::deserialize(deseralizer)? {
        None =>
            Err(<D as Deserializer<'de>>::Error::custom(
                "null value on non-nullable field",
            )),
        some => Ok(some),
    }
}

macro_rules! make_patch {
    (struct $name: ident {
        $($fields: tt)*
    }) => {
        make_patch!(@$name, [], $($fields)*);
    };

    (@$name: ident, [$(($d: expr, $f: ident, $t: ty)),*], $field: ident: Option<$inner_type: ty>, $($fields: tt)*) => {
        make_patch!(@$name, [$(($d, $f, $t),)* ("deserialize_optional", $field, Option<$inner_type>)], $($fields)*);
    };

    (@$name: ident, [$(($d: expr, $f: ident, $t: ty)),*], $field: ident: $inner_type: ty, $($fields: tt)*) => {
        make_patch!(@$name, [$(($d, $f, $t),)* ("deserialize_non_optional", $field, $inner_type)], $($fields)*);
    };

    (@$name: ident, [$(($d: expr, $f: ident, $t: ty)),*], $field: ident: Option<$inner_type: ty>) => {
        make_patch!(@$name, [$(($d, $f, $t),)* ("deserialize_optional", $field, Option<$inner_type>)]);
    };

    (@$name: ident, [$(($d: expr, $f: ident, $t: ty)),*], $field: ident: $inner_type: ty) => {
        make_patch!(@$name, [$(($d, $f, $t),)* ("deserialize_non_optional", $field, $inner_type)]);
    };
    (@$name: ident, [$(($d: expr, $f: ident, $t: ty)),*],) => {
        make_patch!(@$name, [$(($d, $f, $t)),*]);
    };

    (@$name: ident, [$(($deserialize_with: expr, $field: ident, $type: ty)),*]) => {
        #[derive(Deserialize, Debug)]
        pub struct $name {
            $(
                #[serde(default, deserialize_with = $deserialize_with)]
                pub $field: Option<$type>,
            )*
        }
    };
}

macro_rules! patch {
    ($target: expr, $patch: ident: $($field: ident),+) => {
        $(
            if let Some(value) = $patch.$field {
                $target.$field = value;
            }
        )+
    };
}

macro_rules! patch_with {
    ($target: expr, $patch: ident: $($method: ident($field: ident)),+) => {
        $(
            if let Some(value) = $patch.$field {
                $target.$method(value);
            }
        )+
    };

    ($target: expr, $patch: ident: $($method: ident(&$field: ident)),+) => {
        $(
            if let Some(ref value) = $patch.$field {
                $target.$method(value);
            }
        )+
    };
}

macro_rules! map_patch {
    ($target: expr, $patch: ident: $($map: expr => $field: ident),+) => {
        $(
            if let Some(ref value) = $patch.$field {
                $target.$field = $map(value);
            }
        )+
    };
}

macro_rules! map_patch_with {
    ($target: expr, $patch: ident: $($map: expr => $method: ident(&$field: ident)),+) => {
        $(
            if let Some(ref value) = $patch.$field {
                $target.$method($map(value));
            }
        )+
    };

    ($target: expr, $patch: ident: $($map: expr => $method: ident($field: ident)),+) => {
        $(
            if let Some(value) = $patch.$field {
                $target.$method($map(value));
            }
        )+
    };
}

macro_rules! try_map_patch {
    ($target: expr, $patch: ident: $($map: expr => $field: ident),+) => {
        $(
            if let Some(ref value) = $patch.$field {
                $target.$field = $map(value)?;
            }
        )+
    };
}

macro_rules! try_map_patch_with {
    ($target: expr, $patch: ident: $($map: expr => $method: ident(&$field: ident)),+) => {
        $(
            if let Some(ref value) = $patch.$field {
                $target.$method($map(value)?);
            }
        )+
    };

    ($target: expr, $patch: ident: $($map: expr => $method: ident($field: ident)),+) => {
        $(
            if let Some(value) = $patch.$field {
                $target.$method($map(value)?);
            }
        )+
    };
}

macro_rules! validate {
    ($patch: ident: $($validator: path[$field: ident]),+) => {
        $(
            if let Some(ref mut value) = $patch.$field {
                $validator(value)?
            }
        )+
    }
}

macro_rules! validate_db {
    ($patch: ident, $conn: ident: $($validator: path[$field: ident]),+) => {
        $(
            if let Some(ref mut value) = $patch.$field {
                $validator(value, $conn)?
            }
        )+
    }
}

macro_rules! validate_nullable {
    ($patch: ident: $($validator: path[$field: ident]),+) => {
        $(
            if let Some(Some(ref mut value)) = $patch.$field {
                $validator(value)?
            }
        )+
    }
}
