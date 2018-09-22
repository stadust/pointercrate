use actix::Addr;
use crate::actor::{demonlist::DatabaseActor, gdcf::GdcfActor};
use hyper::{
    client::{Client, HttpConnector},
    Body, Request,
};
use hyper_tls::HttpsConnector;
use log::{debug, error, info};
use std::sync::Arc;
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

impl Http {
    pub fn from_env() -> Self {
        Http {
            http_client: Client::builder().build(HttpsConnector::new(4).unwrap()),
            discord_webhook_url: Arc::new(std::env::var("DISCORD_WEBHOOK").ok()),
        }
    }

    pub fn execute_discord_webhook(&self, data: serde_json::Value) -> impl Future<Item = (), Error = ()> {
        if let Some(ref uri) = *self.discord_webhook_url {
            info!("Executing discord webhook!");

            let request = Request::post(uri)
                .header("Content-Type", "application/json")
                .body(Body::from(data.to_string()))
                .unwrap();

            let future = self
                .http_client
                .request(request)
                .map_err(move |error| error!("INTERNAL SERVER ERROR: Failure to execute discord webhook: {:?}", error))
                .map(|_| debug!("Successfully executed discord webhook"));

            Either::A(future)
        } else {
            Either::B(result(Ok(())))
        }
    }

    /// Creates a future that resolves to `()` if a `HEAD` request to the given URL receives a
    /// non-error response status code.
    pub fn if_exists(&self, url: &String) -> impl Future<Item = (), Error = ()> {
        debug!("Verifying {} response to HEAD request with successful status code", url);

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
