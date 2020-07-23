use crate::{
    extractor::{auth::TokenAuth, if_match::IfMatch},
    model::demonlist::submitter::{PatchSubmitter, Submitter, SubmitterPagination},
    permissions::Permissions,
    state::PointercrateState,
    util::HttpResponseBuilderExt,
    ApiResult,
};
use actix_web::{
    web::{Json, Path, Query},
    HttpResponse,
};
use actix_web_codegen::{get, patch};

#[get("/")]
pub async fn paginate(
    TokenAuth(user): TokenAuth, state: PointercrateState, mut pagination: Query<SubmitterPagination>,
) -> ApiResult<HttpResponse> {
    user.inner().require_permissions(Permissions::ListAdministrator)?;

    let mut connection = state.connection().await?;

    let mut submitters = pagination.page(&mut connection).await?;

    let (max_id, min_id) = Submitter::extremal_submitter_ids(&mut connection).await?;

    pagination_response!(
        "/api/v1/submitters/",
        submitters,
        pagination,
        min_id,
        max_id,
        before_id,
        after_id,
        id
    )
}

#[get("/{submitter_id}/")]
pub async fn get(TokenAuth(user): TokenAuth, state: PointercrateState, submitter_id: Path<i32>) -> ApiResult<HttpResponse> {
    user.inner().require_permissions(Permissions::ListModerator)?;

    let mut connection = state.connection().await?;

    let submitter = Submitter::by_id(submitter_id.into_inner(), &mut connection).await?;

    Ok(HttpResponse::Ok().json_with_etag(&submitter))
}

#[patch("/{submitter_id}/")]
pub async fn patch(
    if_match: IfMatch, TokenAuth(user): TokenAuth, state: PointercrateState, submitter_id: Path<i32>, patch: Json<PatchSubmitter>,
) -> ApiResult<HttpResponse> {
    user.inner().require_permissions(Permissions::ListModerator)?;

    let mut connection = state.audited_transaction(&user).await?;

    let submitter = Submitter::by_id(submitter_id.into_inner(), &mut connection).await?;

    if_match.require_etag_match(&submitter)?;

    let submitter = submitter.apply_patch(patch.into_inner(), &mut connection).await?;

    connection.commit().await?;

    Ok(HttpResponse::Ok().json_with_etag(&submitter))
}
