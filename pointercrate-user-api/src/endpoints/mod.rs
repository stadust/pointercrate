use crate::auth::{BasicAuth, TokenAuth};
use pointercrate_core::{error::CoreError, etag::Taggable};
use pointercrate_core_api::{
    error::Result,
    etag::{Precondition, Tagged},
};
use pointercrate_user::{error::UserError, PatchMe, Registration, User};
use rocket::{
    http::{Method, Status},
    request::{FromRequest, Outcome},
    response::Responder,
    serde::json::{serde_json, Json},
    Request, Response,
};
use std::{fmt::Debug, net::IpAddr};

#[rocket::post("/", data = "<body>")]
pub async fn register(ip: IpAddr, body: Json<Registration>) {
    todo!()
}

#[rocket::post("/")]
pub async fn login(auth: BasicAuth) {
    todo!()
}

#[rocket::post("/")]
pub async fn invalidate() {
    todo!()
}

#[rocket::get("/")]
pub fn get_me(auth: TokenAuth) -> Tagged<User> {
    Tagged(auth.0.into_inner())
}

#[rocket::patch("/", data = "<patch>")]
pub fn patch_me(auth: BasicAuth, patch: Json<PatchMe>) {
    todo!()
}

#[rocket::delete("/")]
pub async fn delete_me(mut auth: BasicAuth, pred: Precondition) -> Result<Status> {
    pred.require_etag_match(auth.0.inner())?;

    auth.0.delete(&mut auth.1).await?;
    auth.1.commit().await.map_err(UserError::from)?;

    Ok(Status::NoContent)
}
