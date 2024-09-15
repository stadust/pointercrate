use crate::ratelimits::DemonlistRatelimits;
use log::{debug, error, warn};
use pointercrate_core::{audit::AuditLogEntry, error::CoreError, pool::PointercratePool};
use pointercrate_core_api::{
    error::Result,
    etag::{Precondition, TaggableExt, Tagged},
    pagination::pagination_response,
    query::Query,
    response::Response2,
};
use pointercrate_demonlist::{
    error::DemonlistError,
    player::claim::PlayerClaim,
    record::{
        audit::RecordModificationData,
        note::{notes_on, NewNote, Note, PatchNote},
        submission_count, FullRecord, MinimalRecordPD, PatchRecord, RecordPagination, RecordStatus, Submission,
    },
    submitter::Submitter,
    LIST_ADMINISTRATOR, LIST_HELPER, LIST_MODERATOR,
};
use pointercrate_user_api::auth::TokenAuth;
use rocket::{http::Status, serde::json::Json, tokio, State};
use sqlx::{pool::PoolConnection, Postgres};
use std::net::IpAddr;

/// Pagination endpoint for records in case authentication is provided
///
/// Subject to the following constraints
/// + Only users with `LIST_MODERATOR` permissions can filter by submitter.
/// + Only users with `LIST_HELPER` permissions can filter by record status. For all other users,
/// the `status` property defaults to `APPROVED` (although explicitly setting the status to
/// `APPROVED` is allowed, UNLESS we also filter by player and the player we filter by match a
/// verified claim of the user making the request, in which case access to all records is allowed
/// (the `status` property does not get defaulted, and filtering on it is allowed)
#[rocket::get("/")]
pub async fn paginate(mut auth: TokenAuth, query: Query<RecordPagination>) -> Result<Response2<Json<Vec<MinimalRecordPD>>>> {
    let mut pagination = query.0;

    if pagination.submitter.is_some() {
        auth.require_permission(LIST_MODERATOR)?;
    }

    let claim = PlayerClaim::by_user(auth.user.user().id, &mut auth.connection)
        .await?
        .filter(|c| c.verified);

    if (claim.is_none() || claim.map(|c| c.player.id) != pagination.player) && !auth.has_permission(LIST_HELPER) {
        if pagination.status.is_some() && pagination.status != Some(RecordStatus::Approved) {
            return Err(CoreError::MissingPermissions { required: LIST_HELPER }.into());
        }

        pagination.status = Some(RecordStatus::Approved);
    }

    Ok(pagination_response("/api/v1/records/", pagination, &mut auth.connection).await?)
}

#[rocket::get("/", rank = 1)]
pub async fn unauthed_pagination(
    pool: &State<PointercratePool>, query: Query<RecordPagination>,
) -> Result<Response2<Json<Vec<MinimalRecordPD>>>> {
    let mut connection = pool.connection().await?;
    let mut pagination = query.0;

    if pagination.submitter.is_some() {
        return Err(CoreError::Unauthorized.into());
    }

    if pagination.status.is_some() && pagination.status != Some(RecordStatus::Approved) {
        return Err(CoreError::Unauthorized.into());
    }

    pagination.status = Some(RecordStatus::Approved);

    Ok(pagination_response("/api/v1/records/", pagination, &mut *connection).await?)
}

#[rocket::post("/", data = "<submission>")]
pub async fn submit(
    ip: IpAddr, auth: Option<TokenAuth>, submission: Json<Submission>, pool: &State<PointercratePool>,
    ratelimits: &State<DemonlistRatelimits>,
) -> Result<Response2<Tagged<FullRecord>>> {
    let submission = submission.0;
    let status_is_submitted = submission.status() == RecordStatus::Submitted;
    let (is_team_member, user_id) = match auth {
        Some(ref auth) => (auth.has_permission(LIST_HELPER), Some(auth.user.user().id)),
        None => (false, None),
    };

    if !status_is_submitted || !submission.has_video() {
        match auth {
            Some(ref auth) => auth.require_permission(LIST_HELPER)?,
            None => return Err(CoreError::Unauthorized.into()),
        }
    }

    let mut connection = match auth {
        Some(auth) => auth.connection,
        None => pool.transaction().await?,
    };

    let submitter = match Submitter::by_ip(ip, &mut *connection).await? {
        Some(submitter) => submitter,
        None => {
            ratelimits.new_submitters()?;

            Submitter::create_submitter(ip, &mut *connection).await?
        },
    };

    // Banned submitters cannot submit records
    if submitter.banned {
        return Err(DemonlistError::BannedFromSubmissions.into());
    }

    let normalized = submission.normalize(&mut *connection).await?;

    // check if the player is claimed with submissions locked
    if let Some(claim) = normalized.verified_player_claim(&mut *connection).await? {
        if claim.lock_submissions {
            match user_id {
                Some(user_id) if user_id == claim.user_id => (),
                _ => return Err(DemonlistError::NoThirdPartySubmissions.into()),
            }
        }
    }

    let validated = normalized.validate(&mut *connection).await?;

    if !is_team_member {
        // Check ratelimits before any change is made to the database so that the transaction rollback is
        // easier.

        // Also check the local ratelimit first since that one expires earlier
        ratelimits.record_submission(ip)?;
        ratelimits.record_submission_global()?;
    }

    let mut record = validated.create(submitter, &mut *connection).await?;

    connection.commit().await.map_err(DemonlistError::from)?;

    // FIXME: This is fucking stupid
    if status_is_submitted {
        if let Some(ref video) = record.video {
            tokio::spawn(validate(
                record.id,
                video.to_string(),
                webhook_embed(&record),
                pool.connection().await?,
            ));
        }
    }

    if !is_team_member {
        record.submitter = None;
    }

    let mut response = Response2::tagged(record);

    if status_is_submitted {
        response = response.with_header(
            "X-SUBMISSION-COUNT",
            submission_count(&mut *pool.connection().await?).await?.to_string(),
        );
    }

    Ok(response)
}

#[rocket::get("/<record_id>")]
pub async fn get(record_id: i32, auth: Option<TokenAuth>, pool: &State<PointercratePool>) -> Result<Tagged<FullRecord>> {
    let is_helper = auth.as_ref().is_some_and(|auth| auth.has_permission(LIST_HELPER));

    let mut connection = match auth {
        Some(auth) => auth.connection,
        None => pool.transaction().await?,
    };

    let mut record = FullRecord::by_id(record_id, &mut *connection).await?;

    // TODO: allow access if auth is provided and a verified claim on the record's player is given
    if !is_helper {
        if record.status != RecordStatus::Approved {
            return Err(DemonlistError::RecordNotFound { record_id }.into());
        }
        record.submitter = None;
        record.raw_footage = None;
    }

    Ok(Tagged(record))
}

#[rocket::get("/<record_id>/audit")]
pub async fn audit(record_id: i32, mut auth: TokenAuth) -> Result<Json<Vec<AuditLogEntry<RecordModificationData>>>> {
    auth.require_permission(LIST_ADMINISTRATOR)?;

    let log = pointercrate_demonlist::record::audit::audit_log_for_record(record_id, &mut auth.connection).await?;

    if log.is_empty() {
        return Err(DemonlistError::RecordNotFound { record_id }.into());
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

#[rocket::get("/<record_id>/notes")]
pub async fn get_notes(record_id: i32, mut auth: TokenAuth) -> Result<Response2<Json<Vec<Note>>>> {
    let record_holder_id = sqlx::query!("SELECT player FROM records WHERE id = $1", record_id)
        .fetch_one(&mut *auth.connection)
        .await
        .map_err(|err| {
            if let sqlx::Error::RowNotFound = err {
                DemonlistError::RecordNotFound { record_id }
            } else {
                err.into()
            }
        })?
        .player;

    let notes = if auth.has_permission(LIST_HELPER) {
        notes_on(record_id, false, &mut auth.connection).await?
    } else {
        match PlayerClaim::get(auth.user.user().id, record_holder_id, &mut auth.connection).await {
            Ok(claim) if claim.verified => notes_on(record_id, true, &mut auth.connection).await?,
            Ok(_) | Err(DemonlistError::ClaimNotFound { .. }) => return Err(DemonlistError::RecordNotFound { record_id }.into()),
            Err(err) => return Err(err.into()),
        }
    };

    Ok(Response2::json(notes))
}

#[rocket::post("/<record_id>/notes", data = "<data>")]
pub async fn add_note(record_id: i32, mut auth: TokenAuth, data: Json<NewNote>) -> Result<Response2<Tagged<Note>>> {
    auth.require_permission(LIST_HELPER)?;

    let record = FullRecord::by_id(record_id, &mut auth.connection).await?;

    let mut note = Note::create_on(&record, data.0, &mut auth.connection).await?;

    note.author = Some(auth.user.into_user().name);

    let note_id = note.id;

    auth.connection.commit().await.map_err(DemonlistError::from)?;

    Ok(Response2::tagged(note)
        .status(Status::Created)
        .with_header("Location", format!("/api/v1/records/{}/notes/{}/", record.id, note_id)))
}

#[rocket::patch("/<record_id>/notes/<note_id>", data = "<patch>")]
pub async fn patch_note(record_id: i32, note_id: i32, mut auth: TokenAuth, patch: Json<PatchNote>) -> Result<Tagged<Note>> {
    let note = Note::by_id(record_id, note_id, &mut auth.connection).await?;

    if note.author.as_ref() != Some(&auth.user.user().name) {
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

    if note.author.as_ref() != Some(&auth.user.user().name) {
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

            if (200..400).contains(&status) {
                debug!("GET request yielded some sort of successful response, executing webhook");

                execute_webhook(body).await;
            } else {
                warn!("Server response to 'GET {}' was {:?}, deleting submission!", video, response);

                match FullRecord::delete_by_id(record_id, &mut *connection).await {
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

            match FullRecord::delete_by_id(record_id, &mut *connection).await {
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
