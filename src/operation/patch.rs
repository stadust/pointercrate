use crate::{
    error::PointercrateError, middleware::cond::IfMatch, model::user::Permissions, Result,
};
use diesel::{pg::PgConnection, query_dsl::methods::SelectDsl, Expression, QuerySource};
use serde::{Deserialize, Deserializer};
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

#[derive(Debug)]
pub enum PatchField<T> {
    Null,
    Absent,
    Some(T),
}

impl<T> PatchField<T> {
    pub fn validate<F>(&mut self, f: F) -> Result<()>
    where
        F: FnOnce(T) -> Result<T>,
    {
        // TODO: there's got the be a prettier way for this
        if self.is_some() {
            let t = std::mem::replace(self, PatchField::Null).unwrap();
            std::mem::replace(self, PatchField::Some(f(t)?));
        }

        Ok(())
    }

    fn is_some(&self) -> bool {
        match self {
            PatchField::Some(_) => true,
            _ => false,
        }
    }

    fn unwrap(self) -> T {
        match self {
            PatchField::Some(t) => t,
            _ => panic!(),
        }
    }
}

impl<T> Default for PatchField<T> {
    fn default() -> Self {
        PatchField::Absent
    }
}

pub fn deserialize_patch<'de, T, D>(deserializer: D) -> std::result::Result<PatchField<T>, D::Error>
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
