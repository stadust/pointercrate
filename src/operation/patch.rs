// We're gonna allow unused_macros here because they unused onces are here for completeness sake and
// not having them fucks with my OCD. And we might need them one day if we implement crazy weird
// patch operations, who knows
#![allow(unused_macros)]

use crate::{context::RequestContext, Result};
use diesel::pg::PgConnection;
use serde::{de::Error, Deserialize, Deserializer};
use std::fmt::Display;

pub trait Patch<P>: Display + Sized {
    fn patch(self, patch: P, ctx: RequestContext, connection: &PgConnection) -> Result<Self>;
}

#[allow(clippy::option_option)]
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
        #[allow(clippy::option_option)]
        pub struct $name {
            $(
                #[serde(default, deserialize_with = $deserialize_with)]
                pub $field: Option<$type>,
            )*
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "[ ")?;
                $(
                    if let Some(ref value) = self.$field {
                        write!(f, "{} -> {:?} ", stringify!($field), value)?;
                    }
                )*
                write!(f, "]")
            }
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
                $target.$field = $map(value.as_ref())?;
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

macro_rules! patch_handler {
    ($handler_name: ident, $endpoint: expr, $id_type: ty, $localized_id: expr, $patch_type: ty, $target_type: ty) => {
        /// `PATCH` handler
        pub fn $handler_name(req: &HttpRequest<PointercrateState>) -> PCResponder {
            use crate::middleware::auth::Token;

            info!("PATCH {}", stringify!($endpoint));

            let resource_id = Path::<$id_type>::extract(req).map_err(|_| {
                PointercrateError::bad_request(&format!("{} must be integer", $localized_id))
            });

            let req = req.clone();

            req.json()
                .from_err()
                .and_then(move |patch: $patch_type| Ok((patch, resource_id?.into_inner())))
                .and_then(move |(patch, resource_id)| {
                    req.state()
                        .patch::<Token, _, _, _>(&req, resource_id, patch)
                })
                .map(move |updated: $target_type| HttpResponse::Ok().json_with_etag(updated))
                .responder()
        }
    };

    ($endpoint: expr, $id_type: ty, $localized_id: expr, $patch_type: ty, $target_type: ty) => {
        patch_handler!(
            patch,
            $endpoint,
            $id_type,
            $localized_id,
            $patch_type,
            $target_type
        );
    };
}
