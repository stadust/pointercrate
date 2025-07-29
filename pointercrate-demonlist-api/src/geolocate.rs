use pointercrate_core::pool::PointercratePool;
use pointercrate_core_api::error::IntoOutcome2;
use pointercrate_core_api::{tryo_result, tryo_state};
use pointercrate_demonlist::error::DemonlistError;
use pointercrate_demonlist::nationality::Nationality;
use rocket::request::{FromRequest, Outcome};
use rocket::{async_trait, Request};

#[async_trait]
pub trait GeolocationProvider: Sync + Send {
    /// Geolocates the origin of the given request, returning a tuple of (country, region) iso codes.
    async fn geolocate(&self, req: &Request<'_>) -> Option<(String, Option<String>)>;
}

pub struct GeolocatedNationality(pub Nationality);

#[async_trait]
impl<'r> FromRequest<'r> for GeolocatedNationality {
    type Error = DemonlistError;
    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let geolocator = tryo_state!(request, Box<dyn GeolocationProvider>);

        let Some((country_code, region_iso_code)) = geolocator.geolocate(request).await else {
            return DemonlistError::GeolocationFailed.into_outcome();
        };

        let pool = tryo_state!(request, PointercratePool);
        let mut connection = tryo_result!(pool.connection().await);
        let mut nationality = tryo_result!(Nationality::by_country_code_or_name(&country_code, &mut connection).await);
        if let Some(region) = region_iso_code {
            nationality.subdivision = nationality
                .subdivision_by_code(&region, &mut connection)
                .await
                .inspect_err(|err| {
                    log::warn!(
                        "No subdivision {} for nation {}, or nation does not support subdivisions: {:?}",
                        region,
                        nationality.iso_country_code,
                        err
                    )
                })
                .ok();
        }

        Outcome::Success(GeolocatedNationality(nationality))
    }
}
