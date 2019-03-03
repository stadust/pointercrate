#[macro_use]
mod paginate;
#[macro_use]
mod patch;

pub use self::{
    delete::Delete,
    get::Get,
    paginate::{Paginate, Paginator},
    patch::{deserialize_non_optional, deserialize_optional, Patch},
    post::{Post, PostData},
};

#[macro_use]
mod get {
    use crate::Result;
    use diesel::pg::PgConnection;

    pub trait Get<Key>: Sized {
        fn get(id: Key, connection: &PgConnection) -> Result<Self>;
    }

    impl<G1, G2, Key1, Key2> Get<(Key1, Key2)> for (G1, G2)
    where
        G1: Get<Key1>,
        G2: Get<Key2>,
    {
        fn get((key1, key2): (Key1, Key2), connection: &PgConnection) -> Result<Self> {
            Ok((G1::get(key1, connection)?, G2::get(key2, connection)?))
        }
    }

    impl<G1, G2, G3, Key1, Key2, Key3> Get<(Key1, Key2, Key3)> for (G1, G2, G3)
    where
        G1: Get<Key1>,
        G2: Get<Key2>,
        G3: Get<Key3>,
    {
        fn get((key1, key2, key3): (Key1, Key2, Key3), connection: &PgConnection) -> Result<Self> {
            Ok((
                G1::get(key1, connection)?,
                G2::get(key2, connection)?,
                G3::get(key3, connection)?,
            ))
        }
    }

    macro_rules! get_handler {
        ($handler_name: ident, $endpoint: expr, $id_type: ty, $id_localization: expr, $resource_type: ty) => {
            /// `GET` handler
            pub fn $handler_name(req: &HttpRequest<PointercrateState>) -> PCResponder {
                use crate::middleware::auth::{Authorization, Token};

                info!("GET {}", $endpoint);

                let state = req.state().clone();
                let auth: Authorization = req.extensions_mut().remove().unwrap();

                let resource_id = Path::<$id_type>::extract(req).map_err(|_| {
                    PointercrateError::bad_request(&format!("{} must be integer", $id_localization))
                });

                resource_id
                    .into_future()
                    .and_then(move |resource_id| {
                        state.get::<Token, _, _>(resource_id.into_inner(), auth)
                    })
                    .map(|resource: $resource_type| HttpResponse::Ok().json_with_etag(resource))
                    .responder()
            }
        };

        ($endpoint: expr, $id_type: ty, $id_localization: expr, $resource_type: ty) => {
            get_handler!(get, $endpoint, $id_type, $id_localization, $resource_type);
        };
    }
}

#[macro_use]
mod post {
    use crate::{permissions::PermissionsSet, Result};
    use diesel::pg::PgConnection;

    pub trait Post<T: PostData>: Sized {
        fn create_from(from: T, connection: &PgConnection) -> Result<Self>;
    }

    pub trait PostData {
        fn required_permissions(&self) -> PermissionsSet;
    }

    macro_rules! post_handler_with_authorization {
        ($handler_name: ident, $endpoint: expr, $post_type: ty, $target_type: ty) => {
            /// `POST` handler
            pub fn post(req: &HttpRequest<PointercrateState>) -> PCResponder {
                use crate::middleware::auth::Token;

                info!("POST {}", $endpoint);

                let auth = req.extensions_mut().remove().unwrap();
                let state = req.state().clone();

                req.json()
                    .from_err()
                    .and_then(move |post: $post_type| state.post::<Token, _, _>(post, auth))
                    .map(|created: $target_type| HttpResponse::Created().json_with_etag(created))
                    .responder()
            }
        };

        ($endpoint: expr, $post_type: ty, $target_type: ty) => {
            post_handler_with_authorization!(post, $endpoint, $post_type, $target_type);
        };
    }
}

#[macro_use]
mod delete {
    use crate::{error::PointercrateError, middleware::cond::IfMatch, Result};
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

    macro_rules! delete_handler_with_authorization {
        ($handler_name: ident, $endpoint: expr, $id_type: ty, $id_name: expr, $resource_type: ty) => {
            /// `DELETE` handler
            pub fn $handler_name(req: &HttpRequest<PointercrateState>) -> PCResponder {
                use crate::middleware::{auth::Token, cond::IfMatch};

                info!("DELETE {}", $endpoint);

                let state = req.state().clone();
                let if_match: IfMatch = req.extensions_mut().remove().unwrap();
                let auth = req.extensions_mut().remove().unwrap();

                Path::<$id_type>::extract(req)
                    .map_err(|_| {
                        PointercrateError::bad_request(&format!("{} must be interger", $id_name))
                    })
                    .into_future()
                    .and_then(move |resource_id| {
                        state.delete::<Token, $id_type, $resource_type>(
                            resource_id.into_inner(),
                            Some(if_match),
                            auth,
                        )
                    })
                    .map(|_| HttpResponse::NoContent().finish())
                    .responder()
            }
        };

        ($endpoint: expr, $id_type: ty, $id_name: expr, $resource_type: ty) => {
            delete_handler_with_authorization!(
                delete,
                $endpoint,
                $id_type,
                $id_name,
                $resource_type
            );
        };
    }
}
