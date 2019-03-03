//! Module containing all the actix request handlers for the `/api/v1/records/` endpoints

use super::PCResponder;
use crate::{
    actor::http::PostProcessRecord,
    error::PointercrateError,
    middleware::{auth::Token, cond::HttpResponseBuilderExt},
    model::{
        record::{PatchRecord, Record, RecordPagination, Submission},
        user::User,
        Submitter,
    },
    state::PointercrateState,
};
use actix_web::{AsyncResponder, FromRequest, HttpMessage, HttpRequest, HttpResponse, Path};
use ipnetwork::IpNetwork;
use log::info;
use tokio::prelude::future::{Future, IntoFuture};

// FIXME: we need a prettier way to handle the removal of fields

/// `GET /api/v1/records/` handler
pub fn paginate(req: &HttpRequest<PointercrateState>) -> PCResponder {
    info!("GET /api/v1/records/");

    let query_string = req.query_string();
    let pagination = serde_urlencoded::from_str(query_string)
        .map_err(|err| PointercrateError::bad_request(&err.to_string()));

    let state = req.state().clone();

    state
        /*.authorize(
            req.extensions_mut().remove().unwrap(),
            perms!(ExtendedAccess or ListHelper or ListModerator or ListAdministrator),
        )*/
        .auth::<Token>(req.extensions_mut().remove().unwrap()) // TODO: pagination permissions thingy
        .and_then(move |user| Ok((user.0, pagination?)))
        .and_then(move |(user, pagination): (User, RecordPagination)| {
            state
                .paginate::<Record, _>(pagination, "/api/v1/records/".to_string())
                .and_then(move |(records, links)| {
                    let mut value = serde_json::value::to_value(records)
                        .map_err(PointercrateError::internal)?;
                    let records = value
                        .as_array_mut()
                        .ok_or(PointercrateError::InternalServerError)?;

                    if !user.list_team_member() {
                        records.retain(|record| record["status"] == "approved");

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
    let auth = req.extensions_mut().remove().unwrap();

    req.json()
        .from_err()
        .and_then(move |submission: Submission| {
            state
                .get_internal(remote_addr)
                .and_then(move |submitter: Submitter| {
                    state
                        .post_authorized::<Token, _, _>((submission, submitter), auth)
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

/*pub fn get(req: &HttpRequest<PointercrateState>) -> PCResponder {
    info!("GET {}", "/api/v1/records/[record_id]/");

    let state = req.state().clone();
    let auth = state.authorize(
        req.extensions_mut().remove().unwrap(),
        perms!(ExtendedAccess or ListHelper or ListModerator or ListAdministrator),
    );

    let resource_id = Path::<i32>::extract(req)
        .map_err(|_| PointercrateError::bad_request("Record ID must be integer"));

    resource_id
        .into_future()
        .and_then(move |resource_id| {
            state
                .get(resource_id.into_inner())
                .and_then(|record: Record| {
                    auth.then(move |result| {
                        match result {
                            // List mods see all records with their submitter information
                            Ok(ref user) if user.list_team_member() =>
                                Ok(HttpResponse::Ok().json_with_etag(record)),

                            // Unauthorized people cannot see non-approved records
                            Err(_) if record.status() != RecordStatus::Approved =>
                                Err(PointercrateError::MissingPermissions {required: perms!(ExtendedAccess or ListHelper or ListModerator or ListAdministrator)}),

                            // People with only ExtendedAccess, or unauthorized people (in case of an approved record) will see the record without the submitter info
                            _ => {
                                let mut hasher = DefaultHasher::new();
                                record.hash(&mut hasher);

                                let mut value = serde_json::value::to_value(record)
                                    .map_err(PointercrateError::internal)?;
                                value["submitter"] = serde_json::json!(null);

                                Ok(HttpResponse::Ok()
                                    .header("ETag", hasher.finish().to_string())
                                    .json(serde_json::json!({"data": value})))
                            },
                        }
                    })
                })
        })
        .responder()
}*/

get_handler!("/api/v1/records/[record_id]/", i32, "Record ID", Record);
patch_handler_with_authorization!(
    "/api/v1/records/[record id]/",
    i32,
    "Record ID",
    PatchRecord,
    Record
);
delete_handler_with_authorization!("/api/v1/records/[record id]/", i32, "Record ID", Record);
