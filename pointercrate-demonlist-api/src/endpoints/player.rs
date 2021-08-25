use pointercrate_core::pool::PointercratePool;
use pointercrate_core_api::{
    error::Result,
    etag::{Precondition, TaggableExt, Tagged},
};
use pointercrate_demonlist::player::{FullPlayer, PatchPlayer, Player};
use pointercrate_user_api::auth::TokenAuth;
use rocket::{serde::json::Json, State};

#[rocket::get("/")]
pub async fn paginate() {
    todo!()
}

#[rocket::get("/ranking")]
pub async fn ranking() {
    todo!()
}

#[rocket::get("/<player_id>")]
pub async fn get(player_id: i32, pool: &State<PointercratePool>) -> Result<Tagged<FullPlayer>> {
    let mut connection = pool.connection().await?;

    Ok(Tagged(
        Player::by_id(player_id, &mut connection).await?.upgrade(&mut connection).await?,
    ))
}

#[rocket::patch("/<player_id>", data = "<patch>")]
pub async fn patch(
    player_id: i32, mut auth: TokenAuth, precondition: Precondition, patch: Json<PatchPlayer>,
) -> Result<Tagged<FullPlayer>> {
    let player = Player::by_id(player_id, &mut auth.connection)
        .await?
        .upgrade(&mut auth.connection)
        .await?
        .require_match(precondition)?
        .apply_patch(patch.0, &mut auth.connection)
        .await?;

    auth.commit().await?;

    Ok(Tagged(player))
}
