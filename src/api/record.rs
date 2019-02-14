//! Module containing all the actix request handlers for the `/api/v1/records/` endpoints

use super::PCResponder;
use crate::{
    actor::database::DeleteMessage,
    error::PointercrateError,
    middleware::cond::HttpResponseBuilderExt,
    model::{
        record::{PartialRecord, Record, RecordPagination, Submission},
        Submitter,
    },
    state::PointercrateState,
};
use actix_web::{AsyncResponder, FromRequest, HttpMessage, HttpRequest, HttpResponse, Path};
use ipnetwork::IpNetwork;
use log::{error, info, warn};
use serde_json::json;
use tokio::prelude::future::{Either, Future, IntoFuture};

/// `GET /api/v1/records/` handler
pub fn paginate(req: &HttpRequest<PointercrateState>) -> PCResponder {
    info!("GET /api/v1/records/");

    let query_string = req.query_string();
    let pagination = serde_urlencoded::from_str(query_string)
        .map_err(|err| PointercrateError::bad_request(&err.to_string()));

    let state = req.state().clone();

    state
        .authorize(
            req.extensions_mut().remove().unwrap(),
            perms!(ExtendedAccess or ListHelper or ListModerator or ListAdministrator),
        )
        .and_then(move |_| pagination)
        .and_then(move |pagination: RecordPagination| {
            state.paginate::<PartialRecord, _>(pagination)
        })
        .map(|(records, links)| HttpResponse::Ok().header("Links", links).json(records))
        .responder()
}

/// `POST /api/v1/records/` handler
pub fn submit(req: &HttpRequest<PointercrateState>) -> PCResponder {
    info!("POST /api/v1/records/");

    let state = req.state().clone();
    let state2 = state.clone();
    let remote_addr = req.extensions_mut().remove::<IpNetwork>().unwrap();

    req.json()
        .from_err()
        .and_then(move |submission: Submission| {
            state
                .get(remote_addr)
                .and_then(move |submitter: Submitter| state.post((submission, submitter)))
        })
        .map(|record: Option<Record>| {
            match record {
                Some(record) => {
                    tokio::spawn(post_process_record(&record, state2));

                    HttpResponse::Created().json_with_etag(record)
                },
                None => HttpResponse::NoContent().finish(),
            }
        })
        .responder()
}

get_handler!("/api/v1/records/[record_id]/", i32, "Record ID", Record);
//patch_handler_with_authorization("/api/v1/records/[record id]/", i32, "Record ID", PatchRecord,
// Record);
delete_handler_with_authorization!("/api/v1/records/[record id]/", i32, "Record ID", Record);

fn post_process_record(
    record: &Record, PointercrateState { database, http, .. }: PointercrateState,
) -> impl Future<Item = (), Error = ()> {
    let record_id = record.id;

    let future = if let Some(ref video) = record.video {
        Either::A(http.if_exists(video).or_else(move |_| {
            warn!("A HEAD request to video yielded an error response, automatically deleting submission!");

            database
                .send(DeleteMessage::<i32, Record>::unconditional(record_id))
                .map_err(move |error| error!("INTERNAL SERVER ERROR: Failure to delete record {} - {:?}!", record_id, error))
                .map(|_| ())
                .and_then(|_| Err(()))
        }))
    } else {
        Either::B(Ok(()).into_future())
    };

    let progress = f32::from(record.progress) / 100f32;

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
                "color": (0x9e0000 as f32 * progress) as i32 & 0xFF0000 + (0x00e000 as f32 * progress) as i32 & 0x00FF00,
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
