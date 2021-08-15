use crate::{
    config,
    error::{JsonError, PointercrateError},
    etag::HttpResponseBuilderEtagExt,
    extractor::{auth::TokenAuth, if_match::IfMatch, ip::Ip},
    permissions::Permissions,
    ratelimit::RatelimitScope,
    state::{audit_connection, PointercrateState},
    ApiResult,
};
use actix_web::{
    web::{Json, Path, Query},
    HttpResponse,
};
use actix_web_codegen::{delete, get, patch, post};
use log::{debug, error, warn};
use pointercrate_demonlist::{
    error::DemonlistError,
    record::{
        audit,
        note::{NewNote, Note, PatchNote},
        FullRecord, PatchRecord, RecordPagination, RecordStatus, Submission,
    },
    submitter::Submitter,
};
use serde_json::json;

#[get("/")]
pub async fn paginate(
    user: ApiResult<TokenAuth>, state: PointercrateState, mut pagination: Query<RecordPagination>,
) -> ApiResult<HttpResponse> {
    let mut connection = state.connection().await?;

    // If the submitter or status fields are set (unless status is set to approved and submitter is
    // unset, in which case, proceed), you need to be a list mod. If you aren't authenticated, we return
    // a 401 UNAUTHORIZED, otherwise a 403 FORBIDDEN

    if pagination.submitter.is_some() {
        match user {
            Ok(TokenAuth(ref user)) => user.inner().require_permissions(Permissions::ListModerator)?,
            Err(error) => return Err(error),
        }
    }

    match user {
        Ok(TokenAuth(user)) if user.inner().extended_list_access() => (),
        Ok(TokenAuth(user)) => user.inner().require_permissions(Permissions::ExtendedAccess)?,
        _ =>
            match pagination.status {
                None => pagination.status = Some(RecordStatus::Approved),
                Some(status) if status != RecordStatus::Approved => return Err(JsonError(PointercrateError::Unauthorized)),
                _ => (),
            },
    }

    let mut records = pagination.page(&mut connection).await?;

    let (max_id, min_id) = FullRecord::extremal_record_ids(&mut connection).await?;

    pagination_response!("/api/v1/records/", records, pagination, min_id, max_id, before_id, after_id, id)
}

impl From<DemonlistError> for JsonError {
    fn from(_: DemonlistError) -> Self {
        todo!()
    }
}

#[post("/")]
pub async fn submit(
    Ip(ip): Ip, user: ApiResult<TokenAuth>, submission: Json<Submission>, state: PointercrateState,
) -> ApiResult<HttpResponse> {
    let mut connection = state.transaction().await?;

    // NOTE: don't abort if authentication fails! We might not need it!
    // This prevents invalid auth data in cookies to interfere with record submission

    let shall_ratelimit = user.as_ref().map(|user| !user.0.inner().list_team_member()).unwrap_or(true);

    // only members of the list team can directly add approved records, or add records without video
    if submission.status != RecordStatus::Submitted || submission.video.is_none() {
        // do abort if it fails here!
        let user = user?.0;

        user.inner().require_permissions(Permissions::ListHelper)?;
        audit_connection(&mut connection, user.inner().id).await?; // might as well
    }

    let ratelimiter = state.ratelimits.prepare(ip);

    let submitter = match Submitter::by_ip(ip, &mut connection).await? {
        Some(submitter) => submitter,
        None => {
            ratelimiter.check(RatelimitScope::NewSubmitter)?;

            Submitter::create_submitter(ip, &mut connection).await?
        },
    };

    let validated = submission.0.validate(submitter, &mut connection).await?;

    if shall_ratelimit {
        // Check ratelimits before any change is made to the database so that the transaction rollback is
        // easier.
        ratelimiter.check(RatelimitScope::RecordSubmissionGlobal)?;
        ratelimiter.check(RatelimitScope::RecordSubmission)?;
    }

    let record = validated.create(&mut connection).await?;

    connection.commit().await?;

    let response = HttpResponse::Created()
        .header("Location", format!("/api/v1/records/{}/", record.id))
        .json_with_etag(&record);

    // spawn background task to validate record
    if record.status == RecordStatus::Submitted {
        actix_rt::spawn(validate(record, state));
    }

    Ok(response)
}

#[get("/{record_id}/")]
pub async fn get(user: ApiResult<TokenAuth>, state: PointercrateState, record_id: Path<i32>) -> ApiResult<HttpResponse> {
    let mut connection = state.connection().await?;
    let mut record = FullRecord::by_id(record_id.into_inner(), &mut connection).await?;

    match user {
        Ok(TokenAuth(user)) => {
            if record.status != RecordStatus::Approved {
                user.inner().require_permissions(Permissions::ExtendedAccess)?;
            }
            if !user.inner().has_permission(Permissions::ListHelper) {
                record.notes.clear()
            }
        },
        _ => {
            if record.status != RecordStatus::Approved {
                return Err(JsonError(PointercrateError::Unauthorized))
            }
            record.notes.clear()
        },
    }

    Ok(HttpResponse::Ok().json_with_etag(&record))
}

#[get("/{record_id}/audit/")]
pub async fn audit_log(TokenAuth(user): TokenAuth, state: PointercrateState, record_id: Path<i32>) -> ApiResult<HttpResponse> {
    let mut connection = state.connection().await?;

    user.inner().require_permissions(Permissions::ListHelper)?;

    let record_id = record_id.into_inner();
    let log = audit::entries_for_record(record_id, &mut connection).await?;

    if log.is_empty() {
        Err(PointercrateError::ModelNotFound {
            model: "Record",
            identified_by: record_id.to_string(),
        }
        .into())
    } else {
        Ok(HttpResponse::Ok().json(log))
    }
}

#[patch("/{record_id}/")]
pub async fn patch(
    TokenAuth(user): TokenAuth, if_match: IfMatch, state: PointercrateState, record_id: Path<i32>, data: Json<PatchRecord>,
) -> ApiResult<HttpResponse> {
    let mut connection = state.audited_transaction(&user).await?;

    //user.inner().require_permissions(Permissions::ListHelper)?;

    // FIXME: prevent lost updates by using SELECT ... FOR UPDATE
    let mut record = FullRecord::by_id(record_id.into_inner(), &mut connection).await?;

    if record.demon.position > config::extended_list_size() {
        // only list mods can modify legacy records
        user.inner().require_permissions(Permissions::ListModerator)?;
    } else {
        user.inner().require_permissions(Permissions::ListHelper)?;
    }

    if_match.require_etag_match(&record)?;

    record = record.apply_patch(data.into_inner(), &mut connection).await?;

    connection.commit().await?;

    Ok(HttpResponse::Ok().json_with_etag(&record))
}

#[delete("/{record_id}/")]
pub async fn delete(
    TokenAuth(user): TokenAuth, if_match: IfMatch, state: PointercrateState, record_id: Path<i32>,
) -> ApiResult<HttpResponse> {
    let mut connection = state.audited_transaction(&user).await?;

    // FIXME: prevent lost updates by using SELECT ... FOR UPDATE
    let record = FullRecord::by_id(record_id.into_inner(), &mut connection).await?;

    if record.status == RecordStatus::Submitted {
        if sqlx::query!(
            r#"SELECT EXISTS (SELECT 1 FROM record_modifications WHERE id = $1 AND status_ IS NOT NULL) AS "was_modified!: bool""#,
            record.id
        )
        .fetch_one(&mut *connection)
        .await?
        .was_modified
        {
            user.inner().require_permissions(Permissions::ListModerator)?;
        } else {
            user.inner().require_permissions(Permissions::ListHelper)?;
        }
    } else {
        user.inner().require_permissions(Permissions::ListModerator)?;
    }

    if_match.require_etag_match(&record)?;

    record.delete(&mut connection).await?;

    connection.commit().await?;

    Ok(HttpResponse::NoContent().finish())
}

#[post("/{record_id}/notes/")]
pub async fn add_note(
    TokenAuth(user): TokenAuth, data: Json<NewNote>, record_id: Path<i32>, state: PointercrateState,
) -> ApiResult<HttpResponse> {
    let mut connection = state.audited_connection(&user).await?;

    user.inner().require_permissions(Permissions::ListHelper)?;

    let record = FullRecord::by_id(record_id.into_inner(), &mut connection).await?;
    let mut note = Note::create_on(&record, data.into_inner(), &mut connection).await?;

    note.author = Some(user.into_inner().name);

    Ok(HttpResponse::Created()
        .header("Location", format!("/api/v1/records/{}/notes/{}/", record.id, note.id))
        .json_with_etag(&note))
}

#[patch("/{record_id}/notes/{note_id}/")]
pub async fn patch_note(
    TokenAuth(user): TokenAuth, data: Json<PatchNote>, ids: Path<(i32, i32)>, state: PointercrateState,
) -> ApiResult<HttpResponse> {
    let mut connection = state.audited_transaction(&user).await?;

    let (record_id, note_id) = ids.into_inner();

    let note = Note::by_id(record_id, note_id, &mut connection).await?;

    // Generally you can only modify your own notes
    if note.author.as_ref() != Some(&user.inner().name) {
        user.inner().require_permissions(Permissions::ListAdministrator)?;
    } else {
        user.inner().require_permissions(Permissions::ListHelper)?;
    }

    let note = note.apply_patch(data.into_inner(), &mut connection).await?;

    connection.commit().await?;

    Ok(HttpResponse::Ok().json_with_etag(&note))
}

#[delete("/{record_id}/notes/{note_id}/")]
pub async fn delete_note(TokenAuth(user): TokenAuth, ids: Path<(i32, i32)>, state: PointercrateState) -> ApiResult<HttpResponse> {
    let mut connection = state.audited_transaction(&user).await?;

    let (record_id, note_id) = ids.into_inner();

    let note = Note::by_id(record_id, note_id, &mut connection).await?;

    // Generally you can only delete your own notes
    if note.author.as_ref() != Some(&user.inner().name) {
        user.inner().require_permissions(Permissions::ListAdministrator)?;
    } else {
        user.inner().require_permissions(Permissions::ListHelper)?;
    }

    note.delete(&mut connection).await?;

    connection.commit().await?;

    Ok(HttpResponse::NoContent().finish())
}

async fn validate(record: FullRecord, state: PointercrateState) {
    let mut connection = match state.connection().await {
        Ok(connection) => connection,
        Err(err) => return error!("INTERNAL SERVER ERROR: failed to acquire database connection: {:?}", err),
    };

    let video = match record.video {
        Some(ref video) => video,
        None => return,
    };

    debug!("Verifying that submission {} with video {} actually is valid", record, video);

    match state.http_client.head(video).send().await {
        Ok(response) => {
            let status = response.status().as_u16();

            if status == 401 || status == 403 || status == 405 {
                // Some websites (billibilli) respond unfavorably to HEAD requests. Retry with
                // GET
                match state.http_client.get(video).send().await {
                    Ok(response) => {
                        let status = response.status().as_u16();

                        if status >= 200 && status < 400 {
                            debug!("HEAD request yielded some sort of successful response, executing webhook");

                            execute_webhook(&record, &state).await;
                        }
                    },
                    Err(err) => {
                        error!(
                            "INTERNAL SERVER ERROR: HEAD request to verify video failed: {:?}. Deleting submission",
                            err
                        );

                        match record.delete(&mut connection).await {
                            Ok(_) => (),
                            Err(error) => error!("INTERNAL SERVER ERROR: Failure to delete record - {:?}!", error),
                        }
                    },
                }
            } else if status >= 200 && status < 400 {
                debug!("HEAD request yielded some sort of successful response, executing webhook");

                execute_webhook(&record, &state).await;
            } else {
                warn!("Server response to 'HEAD {}' was {:?}, deleting submission!", video, response);

                match record.delete(&mut connection).await {
                    Ok(_) => (),
                    Err(error) => error!("INTERNAL SERVER ERROR: Failure to delete record - {:?}!", error),
                }
            }
        },
        Err(error) => {
            error!(
                "INTERNAL SERVER ERROR: HEAD request to verify video failed: {:?}. Deleting submission",
                error
            );

            match record.delete(&mut connection).await {
                Ok(_) => (),
                Err(error) => error!("INTERNAL SERVER ERROR: Failure to delete record - {:?}!", error),
            }
        },
    }
}

async fn execute_webhook(record: &FullRecord, state: &PointercrateState) {
    if let Some(ref webhook_url) = state.webhook_url {
        match state
            .http_client
            .post(&**webhook_url)
            .header("Content-Type", "application/json")
            .body(webhook_embed(record).to_string())
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
    let mut payload = json!({
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
        payload["embeds"][0]["fields"] = json! {
            [{
                "name": "Video Proof:",
                "value": video
            }]
        };
    }

    payload
}
