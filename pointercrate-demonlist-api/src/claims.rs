use pointercrate_core::error::CoreError;
use pointercrate_core_api::error::IntoOutcome2;
use pointercrate_core_api::tryo_result;
use pointercrate_demonlist::error::DemonlistError;
use pointercrate_demonlist::player::claim::{ClaimBy, PlayerClaim};
use pointercrate_user_api::auth::Auth;
use rocket::outcome::try_outcome;
use rocket::request::{FromRequest, Outcome};
use rocket::{async_trait, Request};

pub struct AuthWithClaim<T, const VERIFIED: bool>(pub Auth<T>, pub ClaimBy);

#[async_trait]
impl<'r, T, const VERIFIED: bool> FromRequest<'r> for AuthWithClaim<T, VERIFIED>
where
    Auth<T>: FromRequest<'r, Error = CoreError> + Send,
{
    type Error = DemonlistError;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let mut auth = try_outcome!(Auth::<T>::from_request(request)
            .await
            .map_error(|(s, e)| (s, DemonlistError::Core(e))));

        let Some(claim) = tryo_result!(PlayerClaim::by_user(auth.user.user().id, &mut auth.connection).await) else {
            return CoreError::NotFound.into_outcome();
        };

        if VERIFIED && !claim.verified {
            return DemonlistError::ClaimUnverified.into_outcome();
        }

        Outcome::Success(AuthWithClaim(auth, claim))
    }
}
