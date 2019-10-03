use crate::{
    actor::{
        database::{
            Auth, DatabaseActor, DeleteMessage, GetMessage, PaginateMessage, PatchMessage,
            PostMessage,
        },
        http::HttpActor,
    },
    context::RequestData,
    error::PointercrateError,
    middleware::auth::{Authorization, Me, TAuthType},
    operation::{Delete, Get, Paginate, Paginator, Patch, Post},
    Result,
};
use actix::{Addr, Handler, Message};
use actix_web::HttpRequest;
use std::{collections::HashMap, hash::Hash, marker::PhantomData, sync::Arc};
use tokio::prelude::{future::Either, Future};

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
        &self,
        auth: Authorization,
    ) -> impl Future<Item = Me, Error = PointercrateError> {
        self.database(Auth::<T>(auth, PhantomData))
    }

    pub fn get<T, Key, G>(
        &self,
        req: &HttpRequest<Self>,
        key: Key,
    ) -> impl Future<Item = G, Error = PointercrateError>
    where
        T: TAuthType,
        G: Get<Key> + Send + 'static,
        Key: Send + 'static,
    {
        let auth = req.extensions_mut().remove().unwrap();
        let data = RequestData::from_request(req);
        let clone = self.clone();

        match auth {
            Authorization::Unauthorized =>
                Either::A(self.database(GetMessage(key, data, PhantomData))),
            auth =>
                Either::B(self.database(Auth::<T>::new(auth)).and_then(move |user| {
                    clone.database(GetMessage(key, data.with_user(user), PhantomData))
                })),
        }
    }

    pub fn post<A, T, P>(
        &self,
        req: &HttpRequest<Self>,
        t: T,
    ) -> impl Future<Item = P, Error = PointercrateError>
    where
        A: TAuthType,
        T: Send + 'static,
        P: Post<T> + Send + 'static,
    {
        let auth = req.extensions_mut().remove().unwrap();
        let data = RequestData::from_request(req);
        let clone = self.clone();

        match auth {
            Authorization::Unauthorized => Either::A(self.database(PostMessage::new(t, data))),
            auth =>
                Either::B(self.database(Auth::<A>::new(auth)).and_then(move |user| {
                    clone.database(PostMessage::new(t, data.with_user(user)))
                })),
        }
    }

    pub fn delete<T, Key, D>(
        &self,
        req: &HttpRequest<Self>,
        key: Key,
    ) -> impl Future<Item = (), Error = PointercrateError>
    where
        T: TAuthType,
        Key: Send + 'static,
        D: Get<Key> + Delete + Hash + Send + 'static,
    {
        let auth = req.extensions_mut().remove().unwrap();
        let data = RequestData::from_request(req);
        let clone = self.clone();

        match auth {
            Authorization::Unauthorized =>
                Either::A(self.database(DeleteMessage::<Key, D>::new(key, data))),
            auth =>
                Either::B(self.database(Auth::<T>::new(auth)).and_then(move |user| {
                    clone.database(DeleteMessage::<Key, D>::new(key, data.with_user(user)))
                })),
        }
    }

    pub fn patch<T, Key, P, H>(
        &self,
        req: &HttpRequest<Self>,
        key: Key,
        fix: H,
    ) -> impl Future<Item = P, Error = PointercrateError>
    where
        T: TAuthType,
        Key: Send + 'static,
        H: Send + 'static,
        P: Get<Key> + Patch<H> + Send + Hash + 'static,
    {
        let auth = req.extensions_mut().remove().unwrap();
        let data = RequestData::from_request(req);
        let clone = self.clone();

        match auth {
            Authorization::Unauthorized =>
                Either::A(self.database(PatchMessage::new(key, fix, data))),
            auth =>
                Either::B(self.database(Auth::<T>::new(auth)).and_then(move |user| {
                    clone.database(PatchMessage::new(key, fix, data.with_user(user)))
                })),
        }
    }

    pub fn paginate<T, P, D>(
        &self,
        req: &HttpRequest<Self>,
        data: D,
        uri: String,
    ) -> impl Future<Item = (Vec<P>, String), Error = PointercrateError>
    where
        T: TAuthType,
        D: Paginator + Send + 'static,
        P: Paginate<D> + Send + 'static,
    {
        let req_data = RequestData::from_request(req);
        let auth = req.extensions_mut().remove().unwrap();
        let clone = self.clone();

        match auth {
            Authorization::Unauthorized =>
                Either::A(self.database(PaginateMessage(data, uri, req_data, PhantomData))),
            auth =>
                Either::B(self.database(Auth::<T>::new(auth)).and_then(move |user| {
                    clone.database(PaginateMessage(
                        data,
                        uri,
                        req_data.with_user(user),
                        PhantomData,
                    ))
                })),
        }
    }
}
