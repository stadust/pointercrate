use crate::preferences::{ClientPreferences, PreferenceManager};
use crate::{tryo_result, tryo_state};
use pointercrate_core::error::CoreError;
use pointercrate_core::localization::LocaleConfiguration;
use rocket::{
    request::{FromRequest, Outcome},
    Request,
};
use unic_langid::subtags::Language;

pub const LOCALE_COOKIE_NAME: &str = "locale";

pub struct ClientLocale(pub Language);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for ClientLocale {
    type Error = CoreError;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let preference_manager = tryo_state!(request, PreferenceManager);
        let preferences = ClientPreferences::from_cookies(request.cookies(), preference_manager);
        let language = tryo_result!(preferences
            .get(LOCALE_COOKIE_NAME)
            .ok_or_else(|| CoreError::internal_server_error("locale set not registered with preference manager")));
        let lang_id = LocaleConfiguration::get().by_code(language);

        Outcome::Success(ClientLocale(lang_id.language))
    }
}
