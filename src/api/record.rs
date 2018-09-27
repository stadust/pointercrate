use actix_web::{
    AsyncResponder, FromRequest, HttpMessage, HttpRequest, HttpResponse, Path, Responder,
};
use crate::{
    actor::database::{DeleteRecordById, ProcessSubmission, RecordById, SubmitterByIp},
    error::PointercrateError,
    middleware::cond::HttpResponseBuilderExt,
    model::{record::Submission, Record, Submitter},
    state::PointercrateState,
};
use ipnetwork::IpNetwork;
use log::{error, info, warn};
use serde_json::json;
use tokio::prelude::future::{Either, Future, IntoFuture};

pub fn submit(req: &HttpRequest<PointercrateState>) -> impl Responder {
    info!("POST /api/v1/records/");

    let state = req.state().clone();
    let state2 = state.clone();
    let remote_addr = req.extensions_mut().remove::<IpNetwork>().unwrap();

    req.json()
        .from_err()
        .and_then(move |submission: Submission| {
            state
                .database(SubmitterByIp(remote_addr))
                .and_then(move |submitter: Submitter| {
                    state.database(ProcessSubmission(submission, submitter))
                })
        }).map(|record: Option<Record>| {
            match record {
                Some(record) => {
                    tokio::spawn(post_process_record(&record, state2));

                    HttpResponse::Created().json_with_etag(record)
                },
                None => HttpResponse::NoContent().finish(),
            }
        }).responder()
}

pub fn get(req: &HttpRequest<PointercrateState>) -> impl Responder {
    info!("GET /api/v1/records/{{record_id}}/");

    let state = req.state().clone();

    Path::<i32>::extract(req)
        .map_err(|_| PointercrateError::bad_request("Record ID must be integer"))
        .into_future()
        .and_then(move |record_id| state.database(RecordById(record_id.into_inner())))
        .map(|record: Record| HttpResponse::Ok().json_with_etag(record))
        .responder()
}

fn post_process_record(
    record: &Record, PointercrateState { database, http, .. }: PointercrateState,
) -> impl Future<Item = (), Error = ()> {
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
