use crate::claims::AuthWithClaim;
use pointercrate_core::pool::PointercratePool;
use pointercrate_core_api::{
    error::Result,
    etag::{Precondition, TaggableExt, Tagged},
    pagination::pagination_response,
    query::Query,
    response::Response2,
};
use pointercrate_core_macros::localized;
use pointercrate_demonlist::{
    error::DemonlistError,
    player::{
        claim::{ListedClaim, PatchPlayerClaim, PlayerClaim, PlayerClaimPagination},
        DatabasePlayer, FullPlayer, PatchPlayer, Player, PlayerPagination, RankedPlayer, RankingPagination,
    },
    LIST_HELPER,
};
use pointercrate_user::{auth::ApiToken, MODERATOR};
use pointercrate_user_api::auth::Auth;
use rocket::{http::Status, serde::json::Json, State};

#[localized]
#[rocket::get("/")]
pub async fn paginate(
    pool: &State<PointercratePool>, query: Query<PlayerPagination>, auth: Option<Auth<ApiToken>>,
) -> Result<Response2<Json<Vec<Player>>>> {
    let mut pagination = query.0;

    if let Some(auth) = auth {
        if !auth.has_permission(LIST_HELPER) {
            pagination.banned = Some(false);
        }
    } else {
        pagination.banned = Some(false);
    }

    Ok(pagination_response("/api/v1/players/", pagination, &mut *pool.connection().await?).await?)
}

#[localized]
#[rocket::get("/ranking")]
pub async fn ranking(pool: &State<PointercratePool>, query: Query<RankingPagination>) -> Result<Response2<Json<Vec<RankedPlayer>>>> {
    Ok(pagination_response("/api/v1/players/ranking/", query.0, &mut *pool.connection().await?).await?)
}

#[localized]
#[rocket::get("/me", rank = 0)]
pub async fn get_me(auth: AuthWithClaim<ApiToken, false>) -> Result<Tagged<FullPlayer>> {
    let AuthWithClaim(mut auth, claim) = auth;

    let player = Player::by_id(claim.player.id, &mut auth.connection).await?;
    let full_player = player.upgrade(&mut auth.connection).await?;

    Ok(Tagged(full_player))
}

#[localized]
#[rocket::get("/<player_id>")]
pub async fn get(player_id: i32, pool: &State<PointercratePool>) -> Result<Tagged<FullPlayer>> {
    let mut connection = pool.connection().await?;

    Ok(Tagged(
        Player::by_id(player_id, &mut connection).await?.upgrade(&mut connection).await?,
    ))
}

#[localized]
#[rocket::patch("/<player_id>", data = "<patch>")]
pub async fn patch(
    player_id: i32, mut auth: Auth<ApiToken>, precondition: Precondition, patch: Json<PatchPlayer>,
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

#[localized]
#[rocket::put("/<player_id>/claims")]
pub async fn put_claim(player_id: i32, mut auth: Auth<ApiToken>) -> Result<Response2<Json<PlayerClaim>>> {
    let user_id = auth.user.user().id;
    let player = DatabasePlayer::by_id(player_id, &mut auth.connection).await?;
    let claim = player.initiate_claim(user_id, &mut auth.connection).await?;

    auth.commit().await?;

    Ok(Response2::json(claim)
        .status(Status::Created)
        .with_header("Location", format!("/api/v1/players/{}/claims/{}/", player.id, user_id)))
}

/// The `verified` attribute can only be changed by moderator. All other attributes can only be
/// changed by the person holding the claim, but only if the claim is verified (to claim a different
/// player, put in a new `PUT` request)
#[localized]
#[rocket::patch("/<player_id>/claims/<user_id>", data = "<data>")]
pub async fn patch_claim(
    player_id: i32, user_id: i32, mut auth: Auth<ApiToken>, data: Json<PatchPlayerClaim>,
) -> Result<Json<PlayerClaim>> {
    let claim = PlayerClaim::get(user_id, player_id, &mut auth.connection).await;

    if data.verified.is_some() {
        auth.require_permission(MODERATOR)?;
    }

    let claim = match claim {
        Ok(claim) if data.lock_submissions.is_some() => {
            if claim.user_id != auth.user.user().id {
                return Err(DemonlistError::ClaimNotFound {
                    member_id: user_id,
                    player_id,
                }
                .into());
            }

            if !claim.verified {
                return Err(DemonlistError::ClaimUnverified.into());
            }

            claim
        },
        Ok(claim) => claim,
        Err(_) => {
            return Err(DemonlistError::ClaimNotFound {
                member_id: user_id,
                player_id,
            }
            .into())
        },
    };

    let claim = claim.apply_patch(data.0, &mut auth.connection).await?;

    auth.commit().await?;

    Ok(Json(claim))
}

#[localized]
#[rocket::delete("/<player_id>/claims/<user_id>")]
pub async fn delete_claim(player_id: i32, user_id: i32, mut auth: Auth<ApiToken>) -> Result<Status> {
    auth.require_permission(MODERATOR)?;

    let claim = PlayerClaim::get(user_id, player_id, &mut auth.connection).await?;

    claim.delete(&mut auth.connection).await?;
    auth.commit().await?;

    Ok(Status::NoContent)
}

#[localized]
#[rocket::get("/claims")]
pub async fn paginate_claims(
    mut auth: Auth<ApiToken>, pagination: Query<PlayerClaimPagination>,
) -> Result<Response2<Json<Vec<ListedClaim>>>> {
    auth.require_permission(MODERATOR)?;

    Ok(pagination_response("/api/v1/players/claims/", pagination.0, &mut auth.connection).await?)
}

#[cfg(feature = "geolocation")]
#[localized]
#[rocket::post("/me/geolocate")]
pub async fn geolocate_nationality(
    // This is ugly, but there is no other way to trigger our custom error responders from FromRequest impls :/
    auth: std::result::Result<AuthWithClaim<ApiToken, true>, DemonlistError>,
    location: std::result::Result<crate::geolocate::GeolocatedNationality, DemonlistError>,
) -> Result<Json<pointercrate_demonlist::nationality::Nationality>> {
    let AuthWithClaim(mut auth, claim) = auth?;
    let location = location?;

    let mut player = Player::by_id(claim.player.id, &mut auth.connection).await?;

    player.set_nationality(Some(location.0), &mut auth.connection).await?;

    auth.commit().await?;

    Ok(Json(player.nationality.unwrap()))
}
