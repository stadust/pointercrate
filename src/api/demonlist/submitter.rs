use crate::{
    middleware::headers::{HttpRequestExt, HttpResponseBuilderExt},
    model::{
        demonlist::submitter::{FullSubmitter, PatchSubmitter, Submitter, SubmitterPagination},
        user::AuthenticatedUser,
    },
    permissions::Permissions,
    state::PointercrateState,
    Result,
};
use actix_web::{
    web::{Data, Json, Path, Query},
    HttpRequest, HttpResponse,
};
use actix_web_codegen::{get, patch};

#[get("/")]
pub async fn paginate(
    request: HttpRequest, state: Data<PointercrateState>, mut pagination: Query<SubmitterPagination>,
) -> Result<HttpResponse> {
    let mut connection = state.connection().await?;
    let user = AuthenticatedUser::token_auth(request.extensions_mut().remove().unwrap(), &state.secret, &mut connection).await?;

    user.inner().require_permissions(Permissions::ListAdministrator)?;

    let submitters = pagination.page(&mut connection).await?;

    let (max_id, min_id) = Submitter::extremal_submitter_ids(&mut connection).await?;

    pagination_response!(submitters, pagination, min_id, max_id, before_id, after_id)
}

#[get("/{submitter_id}/")]
pub async fn get(request: HttpRequest, state: Data<PointercrateState>, submitter_id: Path<i32>) -> Result<HttpResponse> {
    let mut connection = state.connection().await?;
    let user = AuthenticatedUser::token_auth(request.extensions_mut().remove().unwrap(), &state.secret, &mut connection).await?;

    user.inner().require_permissions(Permissions::ListModerator)?;

    let submitter = FullSubmitter::by_id(submitter_id.into_inner(), &mut connection).await?;

    Ok(HttpResponse::Ok().json_with_etag(submitter))
}

#[patch("/{submitter_id}/")]
pub async fn patch(
    request: HttpRequest, state: Data<PointercrateState>, submitter_id: Path<i32>, patch: Json<PatchSubmitter>,
) -> Result<HttpResponse> {
    let mut connection = state.connection().await?;
    let user = AuthenticatedUser::token_auth(request.extensions_mut().remove().unwrap(), &state.secret, &mut connection).await?;

    user.inner().require_permissions(Permissions::ListModerator)?;

    let submitter = FullSubmitter::by_id(submitter_id.into_inner(), &mut connection).await?;

    request.validate_etag(&submitter)?;

    let submitter = submitter.apply_patch(patch.into_inner(), &mut connection).await?;

    Ok(HttpResponse::Ok().json_with_etag(submitter))
}
