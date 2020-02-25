use crate::{
    middleware::headers::{HttpRequestExt, HttpResponseBuilderExt},
    model::{
        demonlist::{
            record::{FullRecord, PatchRecord, RecordPagination, RecordStatus, Submission},
            submitter::Submitter,
        },
        user::AuthenticatedUser,
    },
    permissions::Permissions,
    ratelimit::RatelimitScope,
    state::PointercrateState,
    Result,
};
use actix_web::{
    web::{Data, Json, Path, Query},
    HttpRequest, HttpResponse,
};
use actix_web_codegen::{delete, get, patch, post};

#[get("/")]
pub async fn paginate(
    request: HttpRequest, state: Data<PointercrateState>, mut pagination: Query<RecordPagination>,
) -> Result<HttpResponse> {
    let mut connection = state.connection().await?;

    // Non authenticated access and access by users without ExtendedAccess perms cannot see non-approved
    // records
    let can_see_all_records =
        if let Ok(user) = AuthenticatedUser::token_auth(request.extensions_mut().remove().unwrap(), &state.secret, &mut connection).await {
            user.inner().extended_list_access()
        } else {
            false
        };

    if !can_see_all_records {
        match pagination.0.status {
            // empty response if we filter by a status != approved and we cant see those records
            Some(status) if status != RecordStatus::Approved => return Ok(HttpResponse::Ok().json(Vec::<()>::new())),
            _ => pagination.0.status = Some(RecordStatus::Approved),
        }
    }

    let records = pagination.page(&mut connection).await?;

    let (max_id, min_id) = FullRecord::extremal_record_ids(&mut connection).await?;

    pagination_response!(records, pagination, min_id, max_id, before_id, after_id)
}

#[post("/")]
pub async fn submit(request: HttpRequest, submission: Json<Submission>, state: Data<PointercrateState>) -> Result<HttpResponse> {
    let mut connection = state.transaction().await?;

    // NOTE: don't abort is authentication fails! We might not need it!
    // This prevents invalid auth data in cookies to interfere with record submission
    let mut user = AuthenticatedUser::token_auth(request.extensions_mut().remove().unwrap(), &state.secret, &mut connection).await;

    // only members of the list team can directly add approved records, or add records without video
    if submission.status != RecordStatus::Submitted || submission.video.is_none() {
        // rewrap the object to prevent borrowing errors. FIXME(ugly)
        let user_ = user?; // do abort if it fails here!
        user_.inner().require_permissions(Permissions::ListHelper)?;
        user = Ok(user_)
    }

    let ip = request.extensions_mut().remove().unwrap();

    let submitter = Submitter::by_ip_or_create(ip, &mut connection).await?;
    let record = FullRecord::create_from(submitter, submission.into_inner(), &mut connection).await?;

    let shall_ratelimit = match user {
        Ok(user) => !user.inner().list_team_member(),
        _ => true,
    };

    // FIXME: this is ugly.
    // At this point the new record is already in the database. However, if we
    // return here, the Transaction object gets dropped, which causes the transaction to be rolled back,
    // so the record addition is reverted
    if shall_ratelimit {
        state.ratelimits.check(RatelimitScope::RecordSubmission, ip)?;
        state.ratelimits.check(RatelimitScope::RecordSubmissionGlobal, ip)?;
    }

    connection.commit().await?;

    Ok(HttpResponse::Created()
        .header("Location", format!("/api/v1/records/{}/", record.id))
        .json_with_etag(record))
}

#[get("/{record_id}/")]
pub async fn get(request: HttpRequest, state: Data<PointercrateState>, record_id: Path<i32>) -> Result<HttpResponse> {
    let mut connection = state.connection().await?;
    let record = FullRecord::by_id(record_id.into_inner(), &mut connection).await?;

    if record.status != RecordStatus::Approved {
        let user = AuthenticatedUser::token_auth(request.extensions_mut().remove().unwrap(), &state.secret, &mut connection).await?;
        user.inner().require_permissions(Permissions::ExtendedAccess)?;
    }

    Ok(HttpResponse::Ok().json_with_etag(record))
}

#[patch("/{record_id}/")]
pub async fn patch(
    request: HttpRequest, state: Data<PointercrateState>, record_id: Path<i32>, data: Json<PatchRecord>,
) -> Result<HttpResponse> {
    let mut connection = state.transaction().await?;

    let user = AuthenticatedUser::token_auth(request.extensions_mut().remove().unwrap(), &state.secret, &mut connection).await?;
    user.inner().require_permissions(Permissions::ListHelper)?;

    // FIXME: prevent lost updates by using SELECT ... FOR UPDATE
    let mut record = FullRecord::by_id(record_id.into_inner(), &mut connection).await?;

    request.validate_etag(&record)?;

    record = record.apply_patch(data.into_inner(), &mut connection).await?;

    connection.commit().await?;

    Ok(HttpResponse::Ok().json_with_etag(record))
}

#[delete("/{record_id}/")]
pub async fn delete(request: HttpRequest, state: Data<PointercrateState>, record_id: Path<i32>) -> Result<HttpResponse> {
    let mut connection = state.transaction().await?;

    let user = AuthenticatedUser::token_auth(request.extensions_mut().remove().unwrap(), &state.secret, &mut connection).await?;
    user.inner().require_permissions(Permissions::ListAdministrator)?;

    // FIXME: prevent lost updates by using SELECT ... FOR UPDATE
    let mut record = FullRecord::by_id(record_id.into_inner(), &mut connection).await?;

    request.validate_etag(&record)?;

    record.delete(&mut connection).await?;

    connection.commit().await?;

    Ok(HttpResponse::NoContent().finish())
}
