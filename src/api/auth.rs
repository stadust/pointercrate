use crate::{
    middleware::headers::{HttpRequestExt, HttpResponseBuilderExt},
    model::user::{AuthenticatedUser, PatchMe, Registration},
    state::PointercrateState,
    Result,
};
use actix_web::{
    web::{Data, Json},
    HttpRequest, HttpResponse, Responder,
};
use actix_web_codegen::{delete, get, patch, post};
use serde_json::json;

#[post("/auth/register/")]
pub async fn register(body: Json<Registration>, state: Data<PointercrateState>) -> Result<HttpResponse> {
    let mut connection = state.connection().await?;
    let user = AuthenticatedUser::register(body.into_inner(), &mut connection).await?;

    Ok(HttpResponse::Created()
        .header("Location", "/api/v1/auth/me/")
        .json_with_etag(user.inner()))
}

#[post("/auth/")]
pub async fn login(request: HttpRequest, state: Data<PointercrateState>) -> Result<HttpResponse> {
    let mut connection = state.connection().await?;
    let authorization = request.extensions_mut().remove().unwrap();

    let user = AuthenticatedUser::basic_auth(&authorization, &mut connection).await?;

    Ok(HttpResponse::Ok().etag(user.inner()).json(json! {{
        "data": user.inner(),
        "token": user.generate_token(&state.secret)
    }}))
}

#[post("/auth/invalidate/")]
pub async fn invalidate(request: HttpRequest, state: Data<PointercrateState>) -> Result<HttpResponse> {
    let mut connection = state.connection().await?;
    let authorization = request.extensions_mut().remove().unwrap();

    AuthenticatedUser::invalidate_all_tokens(authorization, &mut connection).await?;

    Ok(HttpResponse::NoContent().finish())
}

#[get("/auth/me/")]
pub async fn get_me(request: HttpRequest, state: Data<PointercrateState>) -> Result<HttpResponse> {
    let mut connection = state.connection().await?;
    let authorization = request.extensions_mut().remove().unwrap();

    let user = AuthenticatedUser::token_auth(&authorization, &state.secret, &mut connection).await?;

    Ok(HttpResponse::Ok().json_with_etag(user.inner()))
}

#[patch("/auth/me/")]
pub async fn patch_me(request: HttpRequest, state: Data<PointercrateState>, patch: Json<PatchMe>) -> Result<HttpResponse> {
    let mut connection = state.transaction().await?;
    let authorization = request.extensions_mut().remove().unwrap();

    // FIXME: Prevent "Lost Update" by using SELECT ... FOR UPDATE
    let user = AuthenticatedUser::basic_auth(&authorization, &mut connection).await?;

    request.validate_etag(user.inner())?;

    let updated_user = user.apply_patch(patch.into_inner(), &mut connection).await?;

    connection.commit().await?;

    Ok(HttpResponse::Ok().json_with_etag(updated_user.inner()))
}

#[delete("/auth/me/")]
pub async fn delete_me(request: HttpRequest, state: Data<PointercrateState>) -> Result<HttpResponse> {
    let mut connection = state.transaction().await?;
    let authorization = request.extensions_mut().remove().unwrap();

    // FIXME: Prevent "Lost Update" by using SELECT ... FOR UPDATE
    let user = AuthenticatedUser::basic_auth(&authorization, &mut connection).await?;

    request.validate_etag(user.inner())?;

    user.delete(&mut connection).await?;

    connection.commit().await?;

    Ok(HttpResponse::NoContent().finish())
}
