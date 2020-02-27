use crate::{
    extractor::{auth::TokenAuth, if_match::IfMatch},
    model::{
        demonlist::submitter::{FullSubmitter, PatchSubmitter, Submitter, SubmitterPagination},
        user::AuthenticatedUser,
    },
    permissions::Permissions,
    state::PointercrateState,
    util::HttpResponseBuilderExt,
    ApiResult,
};
use actix_web::{
    web::{Data, Json, Path, Query},
    HttpRequest, HttpResponse,
};
use actix_web_codegen::{get, patch};

#[get("/")]
pub async fn paginate(
    TokenAuth(user): TokenAuth, state: PointercrateState, mut pagination: Query<SubmitterPagination>,
) -> ApiResult<HttpResponse> {
    let mut connection = state.connection().await?;

    user.inner().require_permissions(Permissions::ListAdministrator)?;

    let submitters = pagination.page(&mut connection).await?;

    let (max_id, min_id) = Submitter::extremal_submitter_ids(&mut connection).await?;

    pagination_response!(submitters, pagination, min_id, max_id, before_id, after_id, id)
}

#[get("/{submitter_id}/")]
pub async fn get(TokenAuth(user): TokenAuth, state: PointercrateState, submitter_id: Path<i32>) -> ApiResult<HttpResponse> {
    let mut connection = state.connection().await?;

    user.inner().require_permissions(Permissions::ListModerator)?;

    let submitter = FullSubmitter::by_id(submitter_id.into_inner(), &mut connection).await?;

    Ok(HttpResponse::Ok().json_with_etag(&submitter))
}

#[patch("/{submitter_id}/")]
pub async fn patch(
    if_match: IfMatch, TokenAuth(user): TokenAuth, state: PointercrateState, submitter_id: Path<i32>, patch: Json<PatchSubmitter>,
) -> ApiResult<HttpResponse> {
    let mut connection = state.connection().await?;

    user.inner().require_permissions(Permissions::ListModerator)?;

    let submitter = FullSubmitter::by_id(submitter_id.into_inner(), &mut connection).await?;

    if_match.require_etag_match(&submitter)?;

    let submitter = submitter.apply_patch(patch.into_inner(), &mut connection).await?;

    Ok(HttpResponse::Ok().json_with_etag(&submitter))
}
