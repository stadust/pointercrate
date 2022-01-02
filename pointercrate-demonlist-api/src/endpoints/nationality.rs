use pointercrate_core::pool::PointercratePool;
use pointercrate_core_api::{error::Result, etag::Tagged, query::Query};
use pointercrate_demonlist::nationality::{Nationality, NationalityRankingPagination, NationalityRecord, RankedNation, Subdivision};
use rocket::{serde::json::Json, State};

#[rocket::get("/<iso_code>/subdivisions")]
pub async fn subdivisions(pool: &State<PointercratePool>, iso_code: String) -> Result<Json<Vec<Subdivision>>> {
    let mut connection = pool.connection().await?;

    // good code
    let nationality = Nationality::by_country_code_or_name(iso_code.to_uppercase().as_ref(), &mut connection).await?;

    Ok(Json(nationality.subdivisions(&mut connection).await?))
}

#[rocket::get("/ranking")]
pub async fn ranking(pool: &State<PointercratePool>, pagination: Query<NationalityRankingPagination>) -> Result<Json<Vec<RankedNation>>> {
    Ok(Json(pagination.0.page(&mut *pool.connection().await?).await?))
}

#[rocket::get("/<iso_code>")]
pub async fn nation(pool: &State<PointercratePool>, iso_code: String) -> Result<Tagged<NationalityRecord>> {
    let mut connection = pool.connection().await?;

    // good code
    let nationality = Nationality::by_country_code_or_name(iso_code.to_uppercase().as_ref(), &mut connection).await?;

    Ok(Tagged(nationality.upgrade(&mut connection).await?))
}
