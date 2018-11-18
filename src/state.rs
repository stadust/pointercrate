use actix::{Addr, Handler, Message};
use crate::{
    actor::{
        database::{DatabaseActor, DeleteMessage, GetMessage, PatchMessage, PostMessage},
        gdcf::GdcfActor,
    },
    error::PointercrateError,
    middleware::cond::IfMatch,
    model::user::{PermissionsSet, User},
    operation::{Delete, Get, Hotfix, Patch, Post},
    Result,
};
use hyper::{
    client::{Client, HttpConnector},
    Body, Request,
};
use hyper_tls::HttpsConnector;
use log::{debug, error, info};
use std::{hash::Hash, marker::PhantomData, sync::Arc};
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

    pub fn get<Key, G>(&self, key: Key) -> impl Future<Item = G, Error = PointercrateError>
    where
        Key: Send + 'static,
        G: Get<Key> + Send + 'static,
    {
        self.database(GetMessage(key, PhantomData))
    }

    pub fn post<T, P>(&self, t: T) -> impl Future<Item = P, Error = PointercrateError>
    where
        T: Send + 'static,
        P: Post<T> + Send + 'static,
    {
        self.database(PostMessage(t, PhantomData))
    }

    pub fn delete<Key, D>(
        &self, key: Key, condition: IfMatch,
    ) -> impl Future<Item = (), Error = PointercrateError>
    where
        Key: Send + 'static,
        D: Get<Key> + Delete + Hash + Send + 'static,
    {
        self.database(DeleteMessage::<Key, D>(key, Some(condition), PhantomData))
    }

    pub fn patch<Key, P, H>(
        &self, patcher: User, key: Key, fix: H, condition: IfMatch,
    ) -> impl Future<Item = P, Error = PointercrateError>
    where
        Key: Send + 'static,
        H: Hotfix + Send + 'static,
        P: Get<Key> + Patch<H> + Send + Hash + 'static,
    {
        let required_permissions = fix.required_permissions();

        if patcher.permissions() & required_permissions != required_permissions {
            Either::A(result(Err(PointercrateError::MissingPermissions {
                required: PermissionsSet::one(required_permissions),
            })))
        } else {
            Either::B(self.database(PatchMessage(key, fix, Some(condition), PhantomData)))
        }
    }
}

// TODO: we might wanna consider putting the DISCORD_WEBHOOK into a lazy_static.

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
