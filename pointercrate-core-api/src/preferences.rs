use std::collections::HashMap;

use pointercrate_core::error::CoreError;
use pointercrate_core_pages::localization::LocalizationConfiguration;
use rocket::{
    http::CookieJar,
    request::{FromRequest, Outcome},
    Request,
};

/// A request guard which stores the preferences sent from the client.
pub struct ClientPreferences(HashMap<&'static str, String>);

impl ClientPreferences {
    /// Retrieve a particular preference which was sent to us from the client.
    ///
    /// `T` must implement `From<ClientPreference>`, which [`String`] already
    /// implements, in case the untouched cookie value is what needs to be handled.
    pub fn get<T: From<String>>(&self, name: &'static str) -> T {
        T::from(self.0.get(name).cloned().unwrap_or_default())
    }

    pub fn from_cookies(cookies: &CookieJar<'_>, preference_manager: &PreferenceManager) -> Self {
        ClientPreferences(
            preference_manager
                .0
                .iter()
                .map(|(name, default)| {
                    (
                        *name,
                        cookies
                            .get(&format!("preference-{}", name))
                            .map(|cookie| cookie.value().to_string())
                            .unwrap_or(default.to_string()),
                    )
                })
                .collect(),
        )
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for ClientPreferences {
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

/// Represents a preference which this pointercrate instance supports, with a
/// cookie name and a default value.
///
/// Not to be confused with [`ClientPreference`]
pub struct Preference {
    pub name: &'static str,
    pub default: &'static str,
}

/// A configuration state to manage all of your pointercrate instance's
/// client preferences.
#[derive(Default)]
pub struct PreferenceManager(HashMap<&'static str, &'static str>);

impl PreferenceManager {
    /// Append a new preference to this [`PreferenceManager`]. `name` represents
    /// the name of the cookie which stores the value of this preference.
    ///
    /// Note that the cookie name is prefixed with `"preference-"`, so creating a
    /// preference with the `name` value as `"theme"` would result in the cookie
    /// sent from the client being named `"preference-theme"`.
    ///
    /// If the cookie was not received, its value will default to `default`.
    pub fn preference(mut self, name: &'static str, default: &'static str) -> Self {
        self.0.insert(name, default);

        self
    }

    /// Automatically register the preferences needed to store active locales.
    pub fn with_localization_preferences(mut self, config: &LocalizationConfiguration) -> Self {
        self.0.insert(config.default.cookie, config.default.fallback.lang.language.as_str());

        for locale_set in config.overrides.values() {
            self.0.insert(locale_set.cookie, locale_set.fallback.lang.language.as_str());
        }

        self
        //self.preference(config.default.cookie, config.default.fallback.lang.language.as_str())
    }
}
