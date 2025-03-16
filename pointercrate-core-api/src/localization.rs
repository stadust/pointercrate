use pointercrate_core::{error::CoreError, localization::get_locale};
use rocket::{
    request::{FromRequest, Outcome},
    Request,
};
use unic_langid::LanguageIdentifier;

#[derive(Debug)]
pub struct ClientLocale {
    pub code: String,
    pub lang: &'static LanguageIdentifier,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for ClientLocale {
    type Error = CoreError;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let code = request
            .cookies()
            .get("locale")
            .map(|cookie| cookie.value().to_string())
            .unwrap_or("en".to_string()); //todo: this value should be extracted from the specified fallback

        let lang = get_locale(&code);

        Outcome::Success(ClientLocale { code, lang })
    }
}
