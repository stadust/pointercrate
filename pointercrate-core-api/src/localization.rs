use std::path::PathBuf;

use pointercrate_core_pages::localization::LocalizationConfiguration;
use rocket::{
    fs::NamedFile,
    http::{ContentType, Status},
    State,
};

use crate::preferences::ClientPreferences;

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
