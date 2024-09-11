use pointercrate_core_api::{
    error::Result,
    etag::{Precondition, TaggableExt, Tagged},
    pagination::pagination_response,
    query::Query,
    response::Response2,
};
use pointercrate_demonlist::{
    submitter::{PatchSubmitter, Submitter, SubmitterPagination},
    LIST_MODERATOR,
};
use pointercrate_user_api::auth::TokenAuth;
use rocket::serde::json::Json;

#[rocket::get("/")]
pub async fn paginate(mut auth: TokenAuth, pagination: Query<SubmitterPagination>) -> Result<Response2<Json<Vec<Submitter>>>> {
    auth.require_permission(LIST_MODERATOR)?;

    Ok(pagination_response("/api/v1/submitters/", pagination.0, &mut auth.connection).await?)
}

#[rocket::get("/<submitter_id>")]
pub async fn get(submitter_id: i32, mut auth: TokenAuth) -> Result<Tagged<Submitter>> {
    auth.require_permission(LIST_MODERATOR)?;

    Ok(Tagged(Submitter::by_id(submitter_id, &mut auth.connection).await?))
}

#[rocket::patch("/<submitter_id>", data = "<patch>")]
pub async fn patch(
    submitter_id: i32, precondition: Precondition, mut auth: TokenAuth, patch: Json<PatchSubmitter>,
) -> Result<Tagged<Submitter>> {
    auth.require_permission(LIST_MODERATOR)?;

    let submitter = Submitter::by_id(submitter_id, &mut auth.connection)
        .await?
        .require_match(precondition)?
        .apply_patch(patch.0, &mut auth.connection)
        .await?;

    auth.commit().await?;

    Ok(Tagged(submitter))
}
