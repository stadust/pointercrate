use std::path::PathBuf;

use pointercrate_core::{error::CoreError, localization::get_locale};
use pointercrate_core_pages::localization::{Locale, LocalizationConfiguration};
use rocket::{
    fs::NamedFile,
    http::{ContentType, Status},
    request::{FromRequest, Outcome},
    Request, State,
};
use unic_langid::LanguageIdentifier;

use crate::preferences::{ClientPreferences, PreferenceManager};

// Serve our translation files to the frontend
//
// <resource> refers to the translation file name
//
// <uri..> represents the uri of the site for which the translation files are being used
// This allows us to abide by overrides specified in [`LocalizationConfiguration`].
#[rocket::get("/ftl/<resource>/<uri..>")]
pub async fn get_ftl(
    localization_config: &State<LocalizationConfiguration>, preferences: ClientPreferences, resource: &str, uri: PathBuf,
) -> Result<(ContentType, NamedFile), Status> {
    let locale_set = localization_config.set_by_uri(uri);

    let locale = locale_set.by_code(preferences.get::<String>(locale_set.cookie)).unwrap();

    let file = NamedFile::open(format!("locales/{}/{}.ftl", locale.iso_code, resource))
        .await
        .map_err(|_| Status::NotFound)?;

    Ok((ContentType::Plain, file))
}

pub struct ClientLocale(pub Locale);

impl Into<&LanguageIdentifier> for ClientLocale {
    fn into(self) -> &'static LanguageIdentifier {
        get_locale(self.0.iso_code)
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

        let locale_set = localization_config.set_by_uri(PathBuf::from(request.uri().path().as_str()));
        let iso_code: String = preferences.get(locale_set.cookie);

        if let Some(locale) = locale_set.by_code(iso_code) {
            return Outcome::Success(ClientLocale(locale));
        } else {
            return Outcome::Forward(Status::BadRequest);
        }
    }
}
