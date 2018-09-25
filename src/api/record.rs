use actix_web::{AsyncResponder, Error, FromRequest, HttpMessage, HttpRequest, HttpResponse, Path, Responder};
use crate::{
    actor::database::{DeleteRecordById, ProcessSubmission, RecordById, SubmitterByIp},
    error::PointercrateError,
    model::{Record, Submitter},
    state::PointercrateState,
};
use ipnetwork::IpNetwork;
use log::{error, info, warn};
use serde_derive::Deserialize;
use serde_json::json;
use tokio::prelude::future::{Either, Future, IntoFuture};

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

    let state = req.state().clone();
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
                Some(record) => {
                    tokio::spawn(post_process_record(&record, state));

                    HttpResponse::Ok().json(record)
                },
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

fn post_process_record(record: &Record, PointercrateState { database, http, .. }: PointercrateState) -> impl Future<Item = (), Error = ()> {
    let record_id = record.id;

    let future = if let Some(ref video) = record.video {
        Either::A(http.if_exists(video).or_else(move |_| {
            warn!("A HEAD request to video yielded an error response, automatically deleting submission!");

            database
                .send(DeleteRecordById(record_id))
                .map_err(move |error| error!("INTERNAL SERVER ERROR: Failure to delete record {} - {:?}!", record_id, error))
                .map(|_| ())
                .and_then(|_| Err(()))
        }))
    } else {
        Either::B(Ok(()).into_future())
    };

    let payload = json!({
        "content": format!("**New record submitted! ID: {}**", record_id),
        "embeds": [
            {
                "type": "rich",
                "title": format!("{}% on {}", record.progress, record.demon.name),
                "description": format!("{} just got {}% on {}! Go add his record!", record.player.name, record.progress, record.demon.name),
                "footer": {
                    "text": format!("This record has been submitted by submitter #{}", record.submitter)
                },
                "color": (0x9e0000i32 * (record.progress as i32) / 100) & 0xFF0000i32 + (0x00e000i32 * (record.progress as i32) / 100) & 0x00FF00i32,
                "author": {
                    "name": format!("{} (ID: {})", record.player.name, record.player.id),
                    "url": record.video
                },
                "thumbnail": {
                    "url": "https://cdn.discordapp.com/avatars/277391246035648512/b03c85d94dc02084c413a7fdbe2cea79.webp?size=1024"
                },
                "fields": [
                    {
                        "name": "Video Proof:",
                        "value": record.video.as_ref().unwrap_or(&"*None provided!*".to_string())
                    }
                ]
            }
        ]
    });

    future.and_then(move |_| http.execute_discord_webhook(payload))
}
