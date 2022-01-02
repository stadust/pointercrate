use crate::ratelimits::DemonlistRatelimits;
use log::{debug, error, warn};
use pointercrate_core::{audit::AuditLogEntry, error::CoreError, pool::PointercratePool};
use pointercrate_core_api::{
    error::Result,
    etag::{Precondition, TaggableExt, Tagged},
    pagination_response,
    query::Query,
    response::Response2,
};
use pointercrate_demonlist::{
    error::DemonlistError,
    record::{
        audit::RecordModificationData,
        note::{NewNote, Note, PatchNote},
        FullRecord, MinimalRecordPD, PatchRecord, RecordPagination, RecordStatus, Submission,
    },
    submitter::Submitter,
    LIST_ADMINISTRATOR, LIST_HELPER, LIST_MODERATOR,
};
use pointercrate_user_api::auth::TokenAuth;
use rocket::{http::Status, serde::json::Json, tokio, State};
use sqlx::{pool::PoolConnection, Postgres};
use std::net::IpAddr;

#[rocket::get("/")]
pub async fn paginate(mut auth: TokenAuth, query: Query<RecordPagination>) -> Result<Response2<Json<Vec<MinimalRecordPD>>>> {
    let mut pagination = query.0;

    if pagination.submitter.is_some() {
        auth.require_permission(LIST_MODERATOR)?;
    }

    if !auth.has_permission(LIST_HELPER) {
        if pagination.status.is_some() && pagination.status != Some(RecordStatus::Approved) {
            return Err(CoreError::Unauthorized.into())
        }

        pagination.status = Some(RecordStatus::Approved);
    }

    let mut records = pagination.page(&mut auth.connection).await?;

    let (max_id, min_id) = FullRecord::extremal_record_ids(&mut auth.connection).await?;

    pagination_response!("/api/v1/records/", records, pagination, min_id, max_id, before_id, after_id, id)
}

#[rocket::get("/", rank = 1)]
pub async fn unauthed_pagination(
    pool: &State<PointercratePool>, query: Query<RecordPagination>,
) -> Result<Response2<Json<Vec<MinimalRecordPD>>>> {
    let mut connection = pool.connection().await?;
    let mut pagination = query.0;

    if pagination.submitter.is_some() {
        return Err(CoreError::Unauthorized.into())
    }

    if pagination.status.is_some() && pagination.status != Some(RecordStatus::Approved) {
        return Err(CoreError::Unauthorized.into())
    }

    pagination.status = Some(RecordStatus::Approved);

    let mut records = pagination.page(&mut connection).await?;

    let (max_id, min_id) = FullRecord::extremal_record_ids(&mut connection).await?;

    pagination_response!("/api/v1/records/", records, pagination, min_id, max_id, before_id, after_id, id)
}

#[rocket::post("/", data = "<submission>")]
pub async fn submit(
    ip: IpAddr, auth: Option<TokenAuth>, submission: Json<Submission>, pool: &State<PointercratePool>,
    ratelimits: &State<DemonlistRatelimits>,
) -> Result<Tagged<FullRecord>> {
    let submission = submission.0;
    let is_team_member = match auth {
        Some(ref auth) => auth.has_permission(LIST_HELPER),
        None => false,
    };

    if submission.status != RecordStatus::Submitted || submission.video.is_none() {
        match auth {
            Some(ref auth) => auth.require_permission(LIST_HELPER)?,
            None => return Err(CoreError::Unauthorized.into()),
        }
    }

    let mut connection = match auth {
        Some(auth) => auth.connection,
        None => pool.transaction().await?,
    };

    let submitter = match Submitter::by_ip(ip, &mut connection).await? {
        Some(submitter) => submitter,
        None => {
            ratelimits.new_submitters()?;

            Submitter::create_submitter(ip, &mut connection).await?
        },
    };

    let validated = submission.validate(submitter, &mut connection).await?;

    if !is_team_member {
        // Check ratelimits before any change is made to the database so that the transaction rollback is
        // easier.

        // Also check the local ratelimit first since that one expires earlier
        ratelimits.record_submission(ip)?;
        ratelimits.record_submission_global()?;
    }

    let record = validated.create(&mut connection).await?;

    connection.commit().await.map_err(DemonlistError::from)?;

    // FIXME: This is fucking stupid
    if record.status == RecordStatus::Submitted {
        if let Some(ref video) = record.video {
            tokio::spawn(validate(
                record.id,
                video.to_string(),
                webhook_embed(&record),
                pool.connection().await?,
            ));
        }
    }

    Ok(Tagged(record))
}

#[rocket::get("/<record_id>")]
pub async fn get(record_id: i32, auth: Option<TokenAuth>, pool: &State<PointercratePool>) -> Result<Tagged<FullRecord>> {
    let is_helper = match auth {
        Some(ref auth) => auth.has_permission(LIST_HELPER),
        _ => false,
    };

    let mut connection = match auth {
        Some(auth) => auth.connection,
        None => pool.transaction().await?,
    };

    let mut record = FullRecord::by_id(record_id, &mut connection).await?;

    if !is_helper {
        record.notes.clear();

        if record.status != RecordStatus::Approved {
            return Err(DemonlistError::RecordNotFound { record_id }.into())
        }
    }

    Ok(Tagged(record))
}

#[rocket::get("/<record_id>/audit")]
pub async fn audit(record_id: i32, mut auth: TokenAuth) -> Result<Json<Vec<AuditLogEntry<RecordModificationData>>>> {
    auth.require_permission(LIST_ADMINISTRATOR)?;

    let log = pointercrate_demonlist::record::audit::audit_log_for_record(record_id, &mut auth.connection).await?;

    if log.is_empty() {
        return Err(DemonlistError::RecordNotFound { record_id }.into())
    }

    Ok(Json(log))
}

#[rocket::patch("/<record_id>", data = "<patch>")]
pub async fn patch(
    record_id: i32, mut auth: TokenAuth, precondition: Precondition, patch: Json<PatchRecord>,
) -> Result<Tagged<FullRecord>> {
    let record = FullRecord::by_id(record_id, &mut auth.connection).await?;

    if record.demon.position > pointercrate_demonlist::config::extended_list_size() {
        auth.require_permission(LIST_MODERATOR)?;
    } else {
        auth.require_permission(LIST_HELPER)?;
    }

    let record = record
        .require_match(precondition)?
        .apply_patch(patch.0, &mut auth.connection)
        .await?;

    auth.commit().await?;

    Ok(Tagged(record))
}

#[rocket::delete("/<record_id>")]
pub async fn delete(record_id: i32, mut auth: TokenAuth, precondition: Precondition) -> Result<Status> {
    let record = FullRecord::by_id(record_id, &mut auth.connection).await?;

    if record.status == RecordStatus::Submitted && !record.was_modified(&mut auth.connection).await? {
        auth.require_permission(LIST_HELPER)?;
    } else {
        auth.require_permission(LIST_MODERATOR)?;
    }

    precondition.require_etag_match(&record)?;

    record.delete(&mut auth.connection).await?;
    auth.commit().await?;

    Ok(Status::NoContent)
}

#[rocket::post("/<record_id>/notes", data = "<data>")]
pub async fn add_note(record_id: i32, mut auth: TokenAuth, data: Json<NewNote>) -> Result<Response2<Tagged<Note>>> {
    auth.require_permission(LIST_HELPER)?;

    let record = FullRecord::by_id(record_id, &mut auth.connection).await?;

    let mut note = Note::create_on(&record, data.0, &mut auth.connection).await?;

    note.author = Some(auth.user.into_inner().name);

    let note_id = note.id;

    auth.connection.commit().await.map_err(DemonlistError::from)?;

    Ok(Response2::tagged(note)
        .status(Status::Created)
        .with_header("Location", format!("/api/v1/records/{}/notes/{}/", record.id, note_id)))
}

#[rocket::patch("/<record_id>/notes/<note_id>", data = "<patch>")]
pub async fn patch_note(record_id: i32, note_id: i32, mut auth: TokenAuth, patch: Json<PatchNote>) -> Result<Tagged<Note>> {
    let note = Note::by_id(record_id, note_id, &mut auth.connection).await?;

    if note.author.as_ref() == Some(&auth.user.inner().name) {
        auth.require_permission(LIST_ADMINISTRATOR)?;
    } else {
        auth.require_permission(LIST_HELPER)?;
    }

    let note = note.apply_patch(patch.0, &mut auth.connection).await?;

    auth.commit().await?;

    Ok(Tagged(note))
}

#[rocket::delete("/<record_id>/notes/<note_id>")]
pub async fn delete_note(record_id: i32, note_id: i32, mut auth: TokenAuth) -> Result<Status> {
    let note = Note::by_id(record_id, note_id, &mut auth.connection).await?;

    if note.author.as_ref() == Some(&auth.user.inner().name) {
        auth.require_permission(LIST_ADMINISTRATOR)?;
    } else {
        auth.require_permission(LIST_HELPER)?;
    }

    note.delete(&mut auth.connection).await?;

    auth.commit().await?;

    Ok(Status::NoContent)
}

async fn validate(record_id: i32, video: String, body: serde_json::Value, mut connection: PoolConnection<Postgres>) {
    debug!("Verifying that submission {} with video {} actually is valid", record_id, video);

    match reqwest::get(&video).await {
        Ok(response) => {
            let status = response.status().as_u16();

            if status >= 200 && status < 400 {
                debug!("GET request yielded some sort of successful response, executing webhook");

                execute_webhook(body).await;
            } else {
                warn!("Server response to 'GET {}' was {:?}, deleting submission!", video, response);

                match FullRecord::delete_by_id(record_id, &mut connection).await {
                    Ok(_) => (),
                    Err(error) => error!("INTERNAL SERVER ERROR: Failure to delete record - {:?}!", error),
                }
            }
        },
        Err(error) => {
            error!(
                "INTERNAL SERVER ERROR: GET request to verify video failed: {:?}. Deleting submission",
                error
            );

            match FullRecord::delete_by_id(record_id, &mut connection).await {
                Ok(_) => (),
                Err(error) => error!("INTERNAL SERVER ERROR: Failure to delete record - {:?}!", error),
            }
        },
    }
}

async fn execute_webhook(body: serde_json::Value) {
    if let Some(ref webhook_url) = crate::config::submission_webhook() {
        match reqwest::Client::new()
            .post(webhook_url)
            .header("Content-Type", "application/json")
            .body(body.to_string())
            .send()
            .await
        {
            Err(error) => error!("INTERNAL SERVER ERROR: Failure to execute discord webhook: {:?}", error),
            Ok(_) => debug!("Successfully executed discord webhook"),
        }
    } else {
        warn!("Trying to execute webhook, though no link was configured!");
    }
}

fn webhook_embed(record: &FullRecord) -> serde_json::Value {
    let mut payload = serde_json::json!({
        "content": format!("**New record submitted! ID: {}**", record.id),
        "embeds": [
            {
                "type": "rich",
                "title": format!("{}% on {}", record.progress, record.demon.name),
                "description": format!("{} just got {}% on {}! Go add their record!", record.player.name, record.progress, record.demon.name),
                "footer": {
                    "text": format!("This record has been submitted by submitter #{}", record.submitter.map(|s|s.id).unwrap_or(1))
                },
                "author": {
                    "name": format!("{} (ID: {})", record.player.name, record.player.id),
                    "url": record.video
                },
                "thumbnail": {
                    "url": "https://cdn.discordapp.com/avatars/277391246035648512/b03c85d94dc02084c413a7fdbe2cea79.webp?size=1024"
                },
            }
        ]
    });

    if let Some(ref video) = record.video {
        payload["embeds"][0]["fields"] = serde_json::json! {
            [{
                "name": "Video Proof:",
                "value": video
            }]
        };
    }

    payload
}
