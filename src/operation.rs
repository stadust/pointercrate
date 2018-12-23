#[macro_use]
mod paginate;
#[macro_use]
mod patch;

pub use self::{
    delete::Delete,
    get::Get,
    paginate::{Paginate, Paginator},
    patch::{deserialize_non_optional, deserialize_optional, Hotfix, Patch},
    post::Post,
};

mod get {
    use crate::Result;
    use diesel::pg::PgConnection;

    pub trait Get<Key>: Sized {
        fn get(id: Key, connection: &PgConnection) -> Result<Self>;
    }
}
mod post {
    use crate::Result;
    use diesel::pg::PgConnection;

    pub trait Post<T>: Sized {
        fn create_from(from: T, connection: &PgConnection) -> Result<Self>;
    }
}

mod delete {
    use crate::{error::PointercrateError, middleware::cond::IfMatch, Result};
    use diesel::pg::PgConnection;
    use std::{
        collections::hash_map::DefaultHasher,
        hash::{Hash, Hasher},
    };

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
}
