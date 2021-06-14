use crate::{cistring::CiString, model::nationality::Nationality, state::PointercrateState, ApiResult};
use actix_web::{web::Path, HttpResponse};
use actix_web_codegen::get;

#[get("/{iso_code}/subdivisions/")]
pub async fn subdivisions(state: PointercrateState, iso_code: Path<String>) -> ApiResult<HttpResponse> {
    let mut connection = state.connection().await?;

    // good code
    let nationality =
        Nationality::by_country_code_or_name(CiString(iso_code.into_inner().to_uppercase()).as_ref(), &mut connection).await?;

    Ok(HttpResponse::Ok().json(nationality.subdivisions(&mut connection).await?))
}
