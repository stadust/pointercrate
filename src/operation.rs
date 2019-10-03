#[macro_use]
mod paginate;
#[macro_use]
mod patch;

pub use self::{
    delete::Delete,
    get::Get,
    paginate::{Paginate, Paginator, PaginatorQuery, TablePaginator},
    patch::{deserialize_non_optional, deserialize_optional, Patch},
    post::Post,
};

#[macro_use]
mod get {
    use crate::{context::RequestContext, Result};

    pub trait Get<Key>: Sized {
        fn get(id: Key, ctx: RequestContext) -> Result<Self>;
    }

    impl<G1, G2, Key1, Key2> Get<(Key1, Key2)> for (G1, G2)
    where
        G1: Get<Key1>,
        G2: Get<Key2>,
    {
        fn get((key1, key2): (Key1, Key2), ctx: RequestContext) -> Result<Self> {
            Ok((G1::get(key1, ctx)?, G2::get(key2, ctx)?))
        }
    }

    impl<G1, G2, G3, Key1, Key2, Key3> Get<(Key1, Key2, Key3)> for (G1, G2, G3)
    where
        G1: Get<Key1>,
        G2: Get<Key2>,
        G3: Get<Key3>,
    {
        fn get((key1, key2, key3): (Key1, Key2, Key3), ctx: RequestContext) -> Result<Self> {
            Ok((
                G1::get(key1, ctx)?,
                G2::get(key2, ctx)?,
                G3::get(key3, ctx)?,
            ))
        }
    }

    macro_rules! get_handler {
        ($handler_name: ident, $endpoint: expr, $id_type: ty, $id_localization: expr, $resource_type: ty) => {
            /// `GET` handler
            pub fn $handler_name(req: &HttpRequest<PointercrateState>) -> PCResponder {
                use crate::middleware::auth::Token;

                info!("GET {}", $endpoint);

                let resource_id = Path::<$id_type>::extract(req).map_err(|_| {
                    PointercrateError::bad_request(&format!("{} must be integer", $id_localization))
                });

                let req = req.clone();

                resource_id
                    .into_future()
                    .and_then(move |resource_id| {
                        req.state()
                            .get::<Token, _, _>(&req, resource_id.into_inner())
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
    use crate::{context::RequestContext, Result};

    pub trait Post<T>: Sized {
        fn create_from(from: T, ctx: RequestContext) -> Result<Self>;
    }
    macro_rules! post_handler {
        ($handler_name: ident, $endpoint: expr, $post_type: ty, $target_type: ty) => {
            /// `POST` handler
            pub fn post(req: &HttpRequest<PointercrateState>) -> PCResponder {
                use crate::middleware::auth::Token;

                info!("POST {}", $endpoint);

                let req = req.clone();

                req.json()
                    .from_err()
                    .and_then(move |post: $post_type| req.state().post::<Token, _, _>(&req, post))
                    .map(|created: $target_type| HttpResponse::Created().json_with_etag(created))
                    .responder()
            }
        };

        ($endpoint: expr, $post_type: ty, $target_type: ty) => {
            post_handler!(post, $endpoint, $post_type, $target_type);
        };
    }
}

#[macro_use]
mod delete {
    use crate::{context::RequestContext, Result};

    use std::fmt::Display;

    pub trait Delete: Display {
        fn delete(self, ctx: RequestContext) -> Result<()>;
    }

    macro_rules! delete_handler {
        ($handler_name: ident, $endpoint: expr, $id_type: ty, $id_name: expr, $resource_type: ty) => {
            /// `DELETE` handler
            pub fn $handler_name(req: &HttpRequest<PointercrateState>) -> PCResponder {
                use crate::middleware::auth::Token;

                info!("DELETE {}", $endpoint);

                let req = req.clone();

                Path::<$id_type>::extract(&req)
                    .map_err(|_| {
                        PointercrateError::bad_request(&format!("{} must be interger", $id_name))
                    })
                    .into_future()
                    .and_then(move |resource_id| {
                        req.state().delete::<Token, $id_type, $resource_type>(
                            &req,
                            resource_id.into_inner(),
                        )
                    })
                    .map(|_| HttpResponse::NoContent().finish())
                    .responder()
            }
        };

        ($endpoint: expr, $id_type: ty, $id_name: expr, $resource_type: ty) => {
            delete_handler!(delete, $endpoint, $id_type, $id_name, $resource_type);
        };
    }
}
