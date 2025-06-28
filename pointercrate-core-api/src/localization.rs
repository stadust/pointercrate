use pointercrate_core::error::CoreError;
use pointercrate_core_pages::localization::{Locale, LocalizationConfiguration};
use rocket::{
    http::Status,
    request::{FromRequest, Outcome},
    Request,
};
use unic_langid::LanguageIdentifier;

use crate::preferences::{ClientPreferences, PreferenceManager};

pub struct ClientLocale(pub Locale);

impl Into<&LanguageIdentifier> for ClientLocale {
    fn into(self) -> &'static LanguageIdentifier {
        self.0.lang
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for ClientLocale {
    type Error = CoreError;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let localization_config = match request.rocket().state::<LocalizationConfiguration>() {
            Some(localization_config) => localization_config,
            _ => return Outcome::Forward(Status::InternalServerError),
        };
        let preference_manager = match request.rocket().state::<PreferenceManager>() {
            Some(preference_manager) => preference_manager,
            _ => return Outcome::Forward(Status::InternalServerError),
        };

        let preferences = ClientPreferences::from_cookies(request.cookies(), preference_manager);
        let locale_set = localization_config.set_by_uri(request.uri().path().segments().collect());
        let locale = locale_set.by_code(&preferences.get::<String>(locale_set.cookie));

        return Outcome::Success(ClientLocale(locale));
    }
}
