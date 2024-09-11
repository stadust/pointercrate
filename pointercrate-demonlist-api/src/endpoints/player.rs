use crate::{config, ratelimits::DemonlistRatelimits};
use log::warn;
use pointercrate_core::{error::CoreError, pool::PointercratePool};
use pointercrate_core_api::{
    error::Result,
    etag::{Precondition, TaggableExt, Tagged},
    pagination::pagination_response,
    query::Query,
    response::Response2,
};
use pointercrate_demonlist::{
    error::DemonlistError,
    nationality::Nationality,
    player::{
        claim::{ListedClaim, PatchPlayerClaim, PlayerClaim, PlayerClaimPagination},
        DatabasePlayer, FullPlayer, PatchPlayer, Player, PlayerPagination, RankedPlayer, RankingPagination,
    },
    LIST_HELPER,
};
use pointercrate_user::MODERATOR;
use pointercrate_user_api::auth::TokenAuth;
use rocket::{http::Status, serde::json::Json, State};
use serde::Deserialize;
use std::net::IpAddr;

#[rocket::get("/")]
pub async fn paginate(
    pool: &State<PointercratePool>, query: Query<PlayerPagination>, auth: Option<TokenAuth>,
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

#[rocket::get("/ranking")]
pub async fn ranking(pool: &State<PointercratePool>, query: Query<RankingPagination>) -> Result<Response2<Json<Vec<RankedPlayer>>>> {
    Ok(pagination_response("/api/v1/players/ranking/", query.0, &mut *pool.connection().await?).await?)
}

#[rocket::get("/<player_id>")]
pub async fn get(player_id: i32, pool: &State<PointercratePool>) -> Result<Tagged<FullPlayer>> {
    let mut connection = pool.connection().await?;

    Ok(Tagged(
        Player::by_id(player_id, &mut *connection).await?.upgrade(&mut *connection).await?,
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

#[rocket::put("/<player_id>/claims")]
pub async fn put_claim(player_id: i32, mut auth: TokenAuth) -> Result<Response2<Json<PlayerClaim>>> {
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
#[rocket::patch("/<player_id>/claims/<user_id>", data = "<data>")]
pub async fn patch_claim(player_id: i32, user_id: i32, mut auth: TokenAuth, data: Json<PatchPlayerClaim>) -> Result<Json<PlayerClaim>> {
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

#[rocket::delete("/<player_id>/claims/<user_id>")]
pub async fn delete_claim(player_id: i32, user_id: i32, mut auth: TokenAuth) -> Result<Status> {
    auth.require_permission(MODERATOR)?;

    let claim = PlayerClaim::get(user_id, player_id, &mut auth.connection).await?;

    claim.delete(&mut auth.connection).await?;
    auth.commit().await?;

    Ok(Status::NoContent)
}

#[rocket::get("/claims")]
pub async fn paginate_claims(mut auth: TokenAuth, pagination: Query<PlayerClaimPagination>) -> Result<Response2<Json<Vec<ListedClaim>>>> {
    auth.require_permission(MODERATOR)?;

    Ok(pagination_response("/api/v1/players/claims/", pagination.0, &mut auth.connection).await?)
}

#[derive(Deserialize, Debug)]
struct Security {
    is_vpn: bool,
}

#[derive(Deserialize, Debug)]
struct GeolocationResponse {
    security: Security,
    country_code: String,
    region_iso_code: Option<String>,
}

#[rocket::post("/<player_id>/geolocate")]
pub async fn geolocate_nationality(
    player_id: i32, ip: IpAddr, mut auth: TokenAuth, ratelimits: &State<DemonlistRatelimits>,
) -> Result<Json<Nationality>> {
    let mut player = Player::by_id(player_id, &mut auth.connection).await?;
    let claim = PlayerClaim::get(auth.user.user().id, player_id, &mut auth.connection).await?;

    if !claim.verified {
        return Err(DemonlistError::ClaimUnverified.into());
    }

    ratelimits.geolocate(ip)?;

    let response = reqwest::get(format!(
        "https://ipgeolocation.abstractapi.com/v1/?api_key={}&ip_address={}&fields=security,country_code,region_iso_code",
        config::abstract_api_key().ok_or_else(|| CoreError::internal_server_error("No API key for abstract configured"))?,
        ip
    ))
    .await
    .map_err(|err| CoreError::internal_server_error(format!("Ip Geolocation failed: {}", err)))?;

    let data = response.json::<GeolocationResponse>().await.map_err(|err| {
        CoreError::internal_server_error(format!(
            "Ip Geolocation succeeded, but we could not deserialize the response: {}",
            err
        ))
    })?;

    if data.security.is_vpn {
        return Err(DemonlistError::VpsDetected.into());
    }

    let mut nationality = Nationality::by_country_code_or_name(&data.country_code, &mut auth.connection).await?;
    if let Some(region) = data.region_iso_code {
        nationality.subdivision = nationality
            .subdivision_by_code(&region, &mut auth.connection)
            .await
            .inspect_err(|err| {
                warn!(
                    "No subdivision {} for nation {}, or nation does not support subdivisions: {:?}",
                    region, nationality.iso_country_code, err
                )
            })
            .ok();
    }

    player.set_nationality(Some(nationality), &mut auth.connection).await?;

    auth.commit().await?;

    Ok(Json(player.nationality.unwrap()))
}
