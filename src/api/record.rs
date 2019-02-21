//! Module containing all the actix request handlers for the `/api/v1/records/` endpoints

use super::PCResponder;
use crate::{
    actor::http::PostProcessRecord,
    error::PointercrateError,
    middleware::cond::HttpResponseBuilderExt,
    model::{
        record::{PartialRecord, PatchRecord, Record, RecordPagination, Submission},
        user::User,
        Submitter,
    },
    state::PointercrateState,
};
use actix_web::{AsyncResponder, FromRequest, HttpMessage, HttpRequest, HttpResponse, Path};
use ipnetwork::IpNetwork;
use log::info;
use tokio::prelude::future::{Future, IntoFuture};

/// `GET /api/v1/records/` handler
pub fn paginate(req: &HttpRequest<PointercrateState>) -> PCResponder {
    info!("GET /api/v1/records/");

    let query_string = req.query_string();
    let pagination = serde_urlencoded::from_str(query_string)
        .map_err(|err| PointercrateError::bad_request(&err.to_string()));

    let state = req.state().clone();
    let uri = req.uri().to_string();

    state
        .authorize(
            req.extensions_mut().remove().unwrap(),
            perms!(ExtendedAccess or ListHelper or ListModerator or ListAdministrator),
        )
        .and_then(move |user| Ok((user, pagination?)))
        .and_then(move |(user, pagination): (User, RecordPagination)| {
            state
                .paginate::<PartialRecord, _>(pagination, uri)
                .and_then(move |(records, links)| {
                    let mut value = serde_json::value::to_value(records)
                        .map_err(PointercrateError::internal)?;
                    let records = value
                        .as_array_mut()
                        .ok_or(PointercrateError::InternalServerError)?;

                    if !user.has_any(&perms!(ListHelper or ListModerator or ListAdministrator)) {
                        for record in records.iter_mut() {
                            record["submitter"] = serde_json::json!(null);
                        }
                    }

                    Ok(HttpResponse::Ok().header("Links", links).json(records))
                })
        })
        .responder()
}

/// `POST /api/v1/records/` handler
pub fn submit(req: &HttpRequest<PointercrateState>) -> PCResponder {
    info!("POST /api/v1/records/");

    let state = req.state().clone();
    let remote_addr = req.extensions_mut().remove::<IpNetwork>().unwrap();

    req.json()
        .from_err()
        .and_then(move |submission: Submission| {
            state
                .get(remote_addr)
                .and_then(move |submitter: Submitter| {
                    state
                        .post((submission, submitter))
                        .and_then(move |record: Option<Record>| {
                            state.http(PostProcessRecord(record))
                        })
                })
        })
        .map(|record: Option<Record>| {
            match record {
                Some(record) => HttpResponse::Created().json_with_etag(record),
                None => HttpResponse::NoContent().finish(),
            }
        })
        .responder()
}

get_handler!("/api/v1/records/[record_id]/", i32, "Record ID", Record);
patch_handler_with_authorization!(
    "/api/v1/records/[record id]/",
    i32,
    "Record ID",
    PatchRecord,
    Record
);
delete_handler_with_authorization!("/api/v1/records/[record id]/", i32, "Record ID", Record);
