use crate::{
    actor::{
        database::{
            BasicAuth, DatabaseActor, DeleteMessage, GetMessage, PaginateMessage, PatchMessage,
            PostMessage, TokenAuth,
        },
        gdcf::GdcfActor,
    },
    error::PointercrateError,
    middleware::{auth::Authorization, cond::IfMatch},
    model::{user::User, Model},
    operation::{
        Delete, DeletePermissions, Get, GetPermissions, Hotfix, Paginate, Paginator, Patch, Post,
        PostData,
    },
    permissions::PermissionsSet,
    Result,
};
use actix::{Addr, Handler, Message};
use diesel::{
    expression::{AsExpression, NonAggregate},
    pg::Pg,
    query_builder::QueryFragment,
    sql_types::{HasSqlType, NotNull, SqlOrd},
    AppearsOnTable, Expression, QuerySource, SelectableExpression,
};
use hyper::{
    client::{Client, HttpConnector},
    Body, Request,
};
use hyper_tls::HttpsConnector;
use log::{debug, error, info};
use std::{collections::HashMap, hash::Hash, marker::PhantomData, sync::Arc};
use tokio::prelude::future::{result, Either, Future};

#[derive(Debug, Clone)]
pub struct Http {
    pub http_client: Client<HttpsConnector<HttpConnector>>,
    pub discord_webhook_url: Arc<Option<String>>,
}

#[derive(Clone)]
#[allow(missing_debug_implementations)]
pub struct PointercrateState {
    pub database: Addr<DatabaseActor>,
    pub gdcf: Addr<GdcfActor>,
    pub http: Http,

    pub documentation_toc: Arc<String>,
    pub documentation_topics: Arc<HashMap<String, String>>,
}

impl PointercrateState {
    pub fn database<Msg, T>(&self, msg: Msg) -> impl Future<Item = T, Error = PointercrateError>
    where
        T: Send + 'static,
        Msg: Message<Result = Result<T>> + Send + 'static,
        DatabaseActor: Handler<Msg>,
    {
        self.database
            .send(msg)
            .map_err(PointercrateError::internal)
            .flatten()
    }

    pub fn authorize(
        &self, authorization: Authorization, perms: PermissionsSet,
    ) -> impl Future<Item = User, Error = PointercrateError> {
        self.database(TokenAuth(authorization))
            .and_then(move |user| {
                if !user.has_any(&perms) {
                    Err(PointercrateError::MissingPermissions { required: perms })
                } else {
                    Ok(user)
                }
            })
    }

    pub fn authorize_basic(
        &self, authorization: Authorization, perms: PermissionsSet,
    ) -> impl Future<Item = User, Error = PointercrateError> {
        self.database(BasicAuth(authorization))
            .and_then(move |user| {
                if !user.has_any(&perms) {
                    Err(PointercrateError::MissingPermissions { required: perms })
                } else {
                    Ok(user)
                }
            })
    }

    pub fn get_authorized<Key, G>(
        &self, key: Key, authorization: Authorization,
    ) -> impl Future<Item = G, Error = PointercrateError>
    where
        Key: Send + 'static,
        G: Get<Key> + GetPermissions + Send + 'static,
    {
        let clone = self.clone();

        self.authorize(authorization, G::permissions())
            .and_then(move |user| clone.database(GetMessage(key, Some(user), PhantomData)))
    }

    pub fn get<Key, G>(&self, key: Key) -> impl Future<Item = G, Error = PointercrateError>
    where
        Key: Send + 'static,
        G: Get<Key> + Send + 'static,
    {
        self.database(GetMessage(key, None, PhantomData))
    }

    pub fn post_authorized<T, P>(
        &self, t: T, authorization: Authorization,
    ) -> impl Future<Item = P, Error = PointercrateError>
    where
        T: PostData + Send + 'static,
        P: Post<T> + Send + 'static,
    {
        let clone = self.clone();

        self.authorize(authorization, t.required_permissions())
            .and_then(move |user| clone.database(PostMessage(t, Some(user), PhantomData)))
    }

    pub fn post<T, P>(&self, t: T) -> impl Future<Item = P, Error = PointercrateError>
    where
        T: PostData + Send + 'static,
        P: Post<T> + Send + 'static,
    {
        // If this ever happens, its a programming error (should have called `post_authorized`
        // instead) and since its security related, we simply crash the whole server
        assert_eq!(t.required_permissions(), PermissionsSet::default());

        self.database(PostMessage(t, None, PhantomData))
    }

    pub fn delete_authorized<Key, D>(
        &self, key: Key, condition: Option<IfMatch>, authorization: Authorization,
    ) -> impl Future<Item = (), Error = PointercrateError>
    where
        Key: Send + 'static,
        D: Get<Key> + Delete + DeletePermissions + Hash + Send + 'static,
    {
        let clone = self.clone();

        self.authorize(authorization, D::permissions())
            .and_then(move |user| {
                clone.database(DeleteMessage::<Key, D>(
                    key,
                    condition,
                    Some(user),
                    PhantomData,
                ))
            })
    }

    pub fn delete<Key, D>(
        &self, key: Key, condition: IfMatch,
    ) -> impl Future<Item = (), Error = PointercrateError>
    where
        Key: Send + 'static,
        D: Get<Key> + Delete + Hash + Send + 'static,
    {
        self.database(DeleteMessage::<Key, D>(
            key,
            Some(condition),
            None,
            PhantomData,
        ))
    }

    pub fn patch_authorized<Key, P, H>(
        &self, authorization: Authorization, key: Key, fix: H, condition: IfMatch,
    ) -> impl Future<Item = P, Error = PointercrateError>
    where
        Key: Send + 'static,
        H: Hotfix + Send + 'static,
        P: Get<Key> + Patch<H> + Send + Hash + 'static,
    {
        let clone = self.clone();

        self.authorize(authorization, fix.required_permissions())
            .and_then(move |user| {
                clone.database(PatchMessage::new(key, fix, user, Some(condition)))
            })
    }

    pub fn patch_authorized_basic<Key, P, H>(
        &self, authorization: Authorization, key: Key, fix: H, condition: IfMatch,
    ) -> impl Future<Item = P, Error = PointercrateError>
    where
        Key: Send + 'static,
        H: Hotfix + Send + 'static,
        P: Get<Key> + Patch<H> + Send + Hash + 'static,
    {
        let clone = self.clone();

        self.authorize_basic(authorization, fix.required_permissions().into())
            .and_then(move |user| {
                clone.database(PatchMessage::new(key, fix, user, Some(condition)))
            })
    }

    pub fn paginate<P, D>(
        &self, data: D,
    ) -> impl Future<Item = (Vec<P>, String), Error = PointercrateError>
    where
        D: Paginator<Model = P> + Send + 'static,
        P: Paginate<D> + Send + 'static,
        <D::PaginationColumn as Expression>::SqlType: NotNull + SqlOrd,
        <<D::Model as Model>::From as QuerySource>::FromClause: QueryFragment<Pg>,
        Pg: HasSqlType<<D::PaginationColumn as Expression>::SqlType>,
        D::PaginationColumn: SelectableExpression<<D::Model as Model>::From>,
        <D::PaginationColumnType as AsExpression<
            <D::PaginationColumn as Expression>::SqlType,
        >>::Expression: AppearsOnTable<<D::Model as Model>::From>,
        <D::PaginationColumnType as AsExpression<
            <D::PaginationColumn as Expression>::SqlType,
        >>::Expression: NonAggregate,
        <D::PaginationColumnType as AsExpression<
            <D::PaginationColumn as Expression>::SqlType,
        >>::Expression: QueryFragment<Pg>,
    {
        self.database(PaginateMessage(data, PhantomData))
    }
}

impl Http {
    pub fn from_env() -> Self {
        Http {
            http_client: Client::builder().build(HttpsConnector::new(4).unwrap()),
            discord_webhook_url: Arc::new(std::env::var("DISCORD_WEBHOOK").ok()),
        }
    }

    pub fn execute_discord_webhook(
        &self, data: serde_json::Value,
    ) -> impl Future<Item = (), Error = ()> {
        if let Some(ref uri) = *self.discord_webhook_url {
            info!("Executing discord webhook!");

            let request = Request::post(uri)
                .header("Content-Type", "application/json")
                .body(Body::from(data.to_string()))
                .unwrap();

            let future = self
                .http_client
                .request(request)
                .map_err(move |error| {
                    error!(
                        "INTERNAL SERVER ERROR: Failure to execute discord webhook: {:?}",
                        error
                    )
                })
                .map(|_| debug!("Successfully executed discord webhook"));

            Either::A(future)
        } else {
            Either::B(result(Ok(())))
        }
    }

    /// Creates a future that resolves to `()` if a `HEAD` request to the given URL receives a
    /// non-error response status code.
    pub fn if_exists(&self, url: &str) -> impl Future<Item = (), Error = ()> {
        debug!(
            "Verifying {} response to HEAD request with successful status code",
            url
        );

        let request = Request::head(url).body(Body::empty()).unwrap();

        self.http_client
            .request(request)
            .map_err(|error| error!("INTERNAL SERVER ERROR: HEAD request failed: {:?}", error))
            .and_then(|response| {
                let status = response.status().as_u16();

                if 200 <= status && status < 400 {
                    Ok(())
                } else {
                    Err(())
                }
            })
    }
}
