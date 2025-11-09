use std::collections::HashMap;

use crate::localization::LOCALE_COOKIE_NAME;
use pointercrate_core::error::CoreError;
use pointercrate_core::localization::LocaleConfiguration;
use rocket::{
    http::CookieJar,
    request::{FromRequest, Outcome},
    Request,
};

/// A request guard which stores the preferences sent from the client.
pub struct ClientPreferences<'k, 'v>(HashMap<&'k str, &'v str>);

impl<'k: 'v, 'v> ClientPreferences<'k, 'v> {
    /// Retrieve a particular preference which was sent to us from the client.
    ///
    /// `T` must implement `From<ClientPreference>`, which [`String`] already
    /// implements, in case the untouched cookie value is what needs to be handled.
    pub fn get(&self, name: &'k str) -> Option<&'v str> {
        self.0.get(name).copied()
    }

    pub fn from_cookies(cookies: &'v CookieJar<'v>, preference_manager: &'k PreferenceManager) -> Self {
        ClientPreferences(
            preference_manager
                .0
                .iter()
                .map(|(name, default)| {
                    (
                        name.as_ref(),
                        cookies
                            .get(&format!("preference-{}", name))
                            .map(|cookie| cookie.value())
                            .unwrap_or(default),
                    )
                })
                .collect(),
        )
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for ClientPreferences<'r, 'r> {
    type Error = CoreError;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let preference_manager = match request.rocket().state::<PreferenceManager>() {
            Some(preference_manager) => preference_manager,
            _ => return Outcome::Success(ClientPreferences(HashMap::new())), // return an empty preferences hashmap if this instance doesnt support preferences
        };

        let preferences = ClientPreferences::from_cookies(request.cookies(), preference_manager);

        Outcome::Success(preferences)
    }
}

/// A configuration state to manage all of your pointercrate instance's
/// client preferences.
#[derive(Default)]
pub struct PreferenceManager(HashMap<String, String>);

impl PreferenceManager {
    /// Append a new preference to this [`PreferenceManager`]. `name` represents
    /// the name of the cookie which stores the value of this preference.
    ///
    /// Note that the cookie name is prefixed with `"preference-"`, so creating a
    /// preference with the `name` value as `"theme"` would result in the cookie
    /// sent from the client being named `"preference-theme"`.
    ///
    /// If the cookie was not received, its value will default to `default`.
    pub fn preference(mut self, name: impl Into<String>, default: impl Into<String>) -> Self {
        self.0.insert(name.into(), default.into());

        self
    }

    /// Automatically register the preferences needed to store active locales.
    ///
    /// Requires the global localization context to have been set up via [`LocalesLoader::commit`],
    /// otherwise will panic.
    pub fn with_localization(self) -> Self {
        self.preference(LOCALE_COOKIE_NAME, LocaleConfiguration::get().fallback.as_str())
    }
}
