use crate::{
    cistring::CiString,
    etag::HttpResponseBuilderEtagExt,
    model::nationality::{Nationality, NationalityRankingPagination},
    state::PointercrateState,
    ApiResult,
};
use actix_web::{
    web::{Path, Query},
    HttpResponse,
};
use actix_web_codegen::get;

#[get("/{iso_code}/subdivisions/")]
pub async fn subdivisions(state: PointercrateState, iso_code: Path<String>) -> ApiResult<HttpResponse> {
    let mut connection = state.connection().await?;

    // good code
    let nationality =
        Nationality::by_country_code_or_name(CiString(iso_code.into_inner().to_uppercase()).as_ref(), &mut connection).await?;

    Ok(HttpResponse::Ok().json(nationality.subdivisions(&mut connection).await?))
}

#[get("/ranking/")]
pub async fn ranking(state: PointercrateState, pagination: Query<NationalityRankingPagination>) -> ApiResult<HttpResponse> {
    Ok(HttpResponse::Ok().json(pagination.0.page(&mut *state.connection().await?).await?))
}

#[get("/{iso_code}/")]
pub async fn nation(state: PointercrateState, iso_code: Path<String>) -> ApiResult<HttpResponse> {
    let mut connection = state.connection().await?;

    // good code
    let nationality =
        Nationality::by_country_code_or_name(CiString(iso_code.into_inner().to_uppercase()).as_ref(), &mut connection).await?;

    Ok(HttpResponse::Ok().json_with_etag(&nationality.upgrade(&mut connection).await?))
}
