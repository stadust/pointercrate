use pointercrate_core::{error::CoreError, theme::Theme};
use rocket::request::{FromRequest, Outcome, Request};

use crate::{
    preferences::{ClientPreferences, PreferenceManager},
    tryo_result, tryo_state,
};

pub struct ClientTheme(pub Theme);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for ClientTheme {
    type Error = CoreError;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let preference_manager = tryo_state!(request, PreferenceManager);
        let preferences = ClientPreferences::from_cookies(request.cookies(), preference_manager);
        let theme = tryo_result!(preferences
            .get(Theme::cookie_name())
            .ok_or_else(|| CoreError::internal_server_error("theme not registered with preference manager")));

        Outcome::Success(ClientTheme(Theme::from_cookie(theme)))
    }
}
