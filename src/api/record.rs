use actix_web::{ AsyncResponder, Error, HttpMessage, HttpRequest, Responder};
use crate::{
    actor::demonlist::{ProcessSubmission, SubmitterByIp},
    error::PointercrateError,
    PointercrateState,
};
use ipnetwork::IpNetwork;
use log::info;
use serde_derive::Deserialize;
use tokio::prelude::future::Future;

#[derive(Deserialize, Debug)]
pub struct Submission {
    pub progress: i16,
    pub player: String,
    pub demon: String,
    #[serde(default)]
    pub video: Option<String>,
    #[serde(rename = "check", default)]
    pub verify_only: bool,
}

pub fn submit(req: &HttpRequest<PointercrateState>) -> impl Responder {
    info!("POST /api/v1/records");

    let database = req.state().database.clone();
    let remote_addr = req.extensions_mut().remove::<IpNetwork>().unwrap();

    req.json()
        .from_err()
        .and_then(move |submission: Submission| {
            database
                .send(SubmitterByIp(remote_addr))
                .map_err(|_| PointercrateError::InternalServerError)
                .flatten()
                .and_then(move |submitter| {
                    database
                        .send(ProcessSubmission(submission, submitter))
                        .map_err(|_| PointercrateError::InternalServerError)
                        .flatten()
                })
        }).map(|_| "Hello World")
        .from_err::<Error>()
        .responder()
}
