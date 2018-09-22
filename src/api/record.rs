use actix_web::{AsyncResponder, Error, FromRequest, HttpMessage, HttpRequest, HttpResponse, Path, Responder};
use crate::{
    actor::demonlist::{ProcessSubmission, RecordById, SubmitterByIp},
    error::PointercrateError,
    model::{Record, Submitter},
    PointercrateState,
};
use ipnetwork::IpNetwork;
use log::info;
use serde_derive::Deserialize;
use tokio::prelude::future::{Future, IntoFuture};

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
    info!("POST /api/v1/records/");

    let database = req.state().database.clone();
    let remote_addr = req.extensions_mut().remove::<IpNetwork>().unwrap();

    req.json()
        .from_err()
        .and_then(move |submission: Submission| {
            database
                .send(SubmitterByIp(remote_addr))
                .map_err(PointercrateError::internal)
                .flatten()
                .and_then(move |submitter: Submitter| {
                    database
                        .send(ProcessSubmission(submission, submitter))
                        .map_err(PointercrateError::internal)
                        .flatten()
                })
        }).map(|record: Option<Record>| {
            match record {
                Some(record) => HttpResponse::Ok().json(record),
                None => HttpResponse::NoContent().finish(),
            }
        }).from_err::<Error>()
        .responder()
}

pub fn get(req: &HttpRequest<PointercrateState>) -> impl Responder {
    info!("GET /api/v1/records/{{record_id}}/");

    let database = req.state().database.clone();

    Path::<i32>::extract(req)
        .map_err(|_| PointercrateError::bad_request("Record ID must be integer"))
        .into_future()
        .and_then(move |record_id| {
            database
                .send(RecordById(record_id.into_inner()))
                .map_err(PointercrateError::internal)
        }).flatten()
        .map(|record: Record| HttpResponse::Ok().json(record))
        .from_err::<Error>()
        .responder()
}
