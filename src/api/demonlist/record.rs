use crate::{
    config,
    error::{JsonError, PointercrateError},
    extractor::{auth::TokenAuth, if_match::IfMatch, ip::Ip},
    model::demonlist::{
        record::{
            note::{NewNote, Note, PatchNote},
            FullRecord, PatchRecord, RecordPagination, RecordStatus, Submission,
        },
        submitter::Submitter,
    },
    permissions::Permissions,
    state::{audit_connection, PointercrateState},
    util::HttpResponseBuilderExt,
    ApiResult,
};
use actix_web::{
    web::{Json, Path, Query},
    HttpResponse,
};
use actix_web_codegen::{delete, get, patch, post};

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

    let submitter = Submitter::by_ip_or_create(ip, &mut connection, Some(ratelimiter)).await?;

    let record = if shall_ratelimit {
        FullRecord::create_from(submitter, submission.into_inner(), &mut connection, Some(ratelimiter)).await?
    } else {
        FullRecord::create_from(submitter, submission.into_inner(), &mut connection, None).await?
    };

    connection.commit().await?;

    let response = HttpResponse::Created()
        .header("Location", format!("/api/v1/records/{}/", record.id))
        .json_with_etag(&record);

    // spawn background task to validate record
    if record.status == RecordStatus::Submitted {
        actix_rt::spawn(record.validate(state));
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
        user.inner().require_permissions(Permissions::ListHelper)?;
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

    let note = Note::by_id(note_id, &mut connection).await?;

    // Generally you can only modify your own notes
    if note.author.as_ref() != Some(&user.inner().name) {
        user.inner().require_permissions(Permissions::ListAdministrator)?;
    } else {
        user.inner().require_permissions(Permissions::ListHelper)?;
    }

    if note.record != record_id {
        return Err(PointercrateError::ModelNotFound {
            model: "Note",
            identified_by: format!("{} on record {}", note_id, record_id),
        }
        .into())
    }

    let note = note.apply_patch(data.into_inner(), &mut connection).await?;

    connection.commit().await?;

    Ok(HttpResponse::Ok().json_with_etag(&note))
}

#[delete("/{record_id}/notes/{note_id}/")]
pub async fn delete_note(TokenAuth(user): TokenAuth, ids: Path<(i32, i32)>, state: PointercrateState) -> ApiResult<HttpResponse> {
    let mut connection = state.audited_transaction(&user).await?;

    let (record_id, note_id) = ids.into_inner();

    let note = Note::by_id(note_id, &mut connection).await?;

    // Generally you can only delete your own notes
    if note.author.as_ref() != Some(&user.inner().name) {
        user.inner().require_permissions(Permissions::ListAdministrator)?;
    } else {
        user.inner().require_permissions(Permissions::ListHelper)?;
    }

    if note.record != record_id {
        return Err(PointercrateError::ModelNotFound {
            model: "Note",
            identified_by: format!("{} on record {}", note_id, record_id),
        }
        .into())
    }

    note.delete(&mut connection).await?;

    connection.commit().await?;

    Ok(HttpResponse::NoContent().finish())
}
