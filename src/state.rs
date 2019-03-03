use crate::{
    actor::{
        database::{
            Auth, DatabaseActor, DeleteMessage, GetInternal, GetMessage, PaginateMessage,
            PatchMessage, PostMessage,
        },
        http::HttpActor,
    },
    error::PointercrateError,
    middleware::{
        auth::{Authorization, Me, TAuthType, Token},
        cond::IfMatch,
    },
    model::Model,
    operation::{
        Delete, DeletePermissions, Get, Hotfix, Paginate, Paginator, Patch, Post, PostData,
    },
    permissions::{AccessRestrictions, PermissionsSet},
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
use std::{collections::HashMap, hash::Hash, marker::PhantomData, sync::Arc};
use tokio::prelude::{
    future::{err, Either},
    Future,
};

#[derive(Clone)]
#[allow(missing_debug_implementations)]
pub struct PointercrateState {
    pub database: Addr<DatabaseActor>,
    pub gdcf: Addr<HttpActor>,

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

    pub fn http<Msg, T>(&self, msg: Msg) -> impl Future<Item = T, Error = PointercrateError>
    where
        T: Send + 'static,
        Msg: Message<Result = T> + Send + 'static,
        HttpActor: Handler<Msg>,
    {
        self.gdcf.send(msg).map_err(PointercrateError::internal)
    }

    pub fn auth<T: TAuthType>(
        &self, auth: Authorization,
    ) -> impl Future<Item = Me, Error = PointercrateError> {
        self.database(Auth::<T>(auth, PhantomData))
    }

    pub fn get_unauthorized<Key, G>(
        &self, key: Key,
    ) -> impl Future<Item = G, Error = PointercrateError>
    where
        G: Get<Key> + AccessRestrictions + Send + 'static,
        Key: Send + 'static,
    {
        // Auth type doesnt matter, as we don't do auth
        self.get::<Token, _, _>(key, Authorization::Unauthorized)
    }

    pub fn get<T, Key, G>(
        &self, key: Key, auth: Authorization,
    ) -> impl Future<Item = G, Error = PointercrateError>
    where
        T: TAuthType,
        G: Get<Key> + AccessRestrictions + Send + 'static,
        Key: Send + 'static,
    {
        let clone = self.clone();

        match auth {
            Authorization::Unauthorized =>
                Either::A(self.database(GetMessage(key, None, PhantomData))),
            auth =>
                Either::B(self.database(Auth::<T>::new(auth)).and_then(move |user| {
                    clone.database(GetMessage(key, Some(user.0), PhantomData))
                })),
        }
    }

    pub fn get_internal<Key, G>(&self, key: Key) -> impl Future<Item = G, Error = PointercrateError>
    where
        G: Get<Key> + Send + 'static,
        Key: Send + 'static,
    {
        self.database(GetInternal(key, PhantomData))
    }

    pub fn post_authorized<A, T, P>(
        &self, t: T, auth: Authorization,
    ) -> impl Future<Item = P, Error = PointercrateError>
    where
        A: TAuthType,
        T: PostData + Send + 'static,
        P: Post<T> + Send + 'static,
    {
        let clone = self.clone();

        // Why do it this way around instead of checking `t.required_permissions() ==
        // PermissionsSet::default()` first and simply not doing the whole authorization thing when
        // it's the case? Because this way allows us to pass the user object from a successful
        // authentication to the `PostMessage` struct even if no permissions are required. This
        // means that in those cases we still get audit-log entries
        self.database(Auth::<A>::new(auth)) // TODO: reintroduce permission checking
            .then(move |result| {
                match result {
                    Ok(user) => Either::A(clone.database(PostMessage(t, Some(user.0), PhantomData))),
                    Err(PointercrateError::Unauthorized)
                        if t.required_permissions() == PermissionsSet::default() =>
                        Either::A(clone.database(PostMessage(t, None, PhantomData))),
                    Err(e) => Either::B(err(e)),
                }
            })
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

    pub fn delete_authorized<T, Key, D>(
        &self, key: Key, condition: Option<IfMatch>, auth: Authorization,
    ) -> impl Future<Item = (), Error = PointercrateError>
    where
        T: TAuthType,
        Key: Send + 'static,
        D: Get<Key> + Delete + DeletePermissions + Hash + Send + 'static,
    {
        let clone = self.clone();

        self.database(Auth::<T>::new(auth))// TODO: reintroduce permission checking
            .and_then(move |user| {
                clone.database(DeleteMessage::<Key, D>(
                    key,
                    condition,
                    Some(user.0),
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

    pub fn patch_authorized<T, Key, P, H>(
        &self, auth: Authorization, key: Key, fix: H, condition: IfMatch,
    ) -> impl Future<Item = P, Error = PointercrateError>
    where
        T: TAuthType,
        Key: Send + 'static,
        H: Hotfix + Send + 'static,
        P: Get<Key> + Patch<H> + Send + Hash + 'static,
    {
        let clone = self.clone();

        self.database(Auth::<T>::new(auth)) // TODO: reintroduce permission checking
            .and_then(move |user| {
                clone.database(PatchMessage::new(key, fix, user.0, Some(condition)))
            })
    }

    pub fn paginate<T, P, D>(
        &self, data: D, uri: String, auth: Authorization
    ) -> impl Future<Item = (Vec<P>, String), Error = PointercrateError>
    where
        T: TAuthType,
        D: Paginator<Model = P> + Send + 'static,
        P: Paginate<D> + AccessRestrictions + Send + 'static,
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
        let clone = self.clone();

        match auth {
            Authorization::Unauthorized =>
                Either::A(self.database(PaginateMessage(data, uri, None, PhantomData))),
            auth =>
                Either::B(self.database(Auth::<T>::new(auth)).and_then(move |user| {
                    clone.database(PaginateMessage(data, uri, Some(user.0), PhantomData))
                })),
        }
    }
}
