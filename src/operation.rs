#[macro_use]
mod paginate;
#[macro_use]
mod patch;

pub use self::{
    delete::{Delete, DeletePermissions},
    get::{Get, GetPermissions},
    paginate::{Paginate, Paginator},
    patch::{deserialize_non_optional, deserialize_optional, Hotfix, Patch},
    post::{Post, PostData},
};

mod get {
    use crate::{permissions::PermissionsSet, Result};
    use diesel::pg::PgConnection;

    pub trait Get<Key>: Sized {
        fn get(id: Key, connection: &PgConnection) -> Result<Self>;
    }

    pub trait GetPermissions {
        fn permissions() -> PermissionsSet {
            PermissionsSet::default()
        }
    }
}

mod post {
    use crate::{permissions::PermissionsSet, Result};
    use diesel::pg::PgConnection;

    pub trait Post<T: PostData>: Sized {
        fn create_from(from: T, connection: &PgConnection) -> Result<Self>;
    }

    pub trait PostData {
        fn required_permissions(&self) -> PermissionsSet;
    }
}

mod delete {
    use crate::{
        error::PointercrateError, middleware::cond::IfMatch, permissions::PermissionsSet, Result,
    };
    use diesel::pg::PgConnection;
    use log::info;
    use std::{
        collections::hash_map::DefaultHasher,
        fmt::Display,
        hash::{Hash, Hasher},
    };

    pub trait Delete: Display {
        fn delete(self, connection: &PgConnection) -> Result<()>;

        fn delete_if_match(self, condition: IfMatch, connection: &PgConnection) -> Result<()>
        where
            Self: Hash + Sized,
        {
            info!("Patching {} only if {} is met", self, condition);

            let mut hasher = DefaultHasher::new();
            self.hash(&mut hasher);

            if condition.met(hasher.finish()) {
                self.delete(connection)
            } else {
                Err(PointercrateError::PreconditionFailed)
            }
        }
    }

    pub trait DeletePermissions {
        fn permissions() -> PermissionsSet {
            PermissionsSet::default()
        }
    }
}
