use pointercrate_core::{error::CoreError, localization::get_locale};
use rocket::{
    http::CookieJar,
    request::{FromRequest, Outcome},
    Request,
};
use unic_langid::LanguageIdentifier;

/// Represents a single preference received from the client.
///
/// In order to map the cookie value to a group of particular values, consider
/// creating an enumeration which implements `From<ClientPreference>`.
///
/// # Example
/// ```ignore
/// enum ClientTheme {
///     Light,
///     Dark,
/// }
///
/// impl From<ClientPreference> for ClientTheme {
///     fn from(preference: ClientPreference) -> Self {
///         match preference.value.as_str() {
///             "light" => ClientTheme::Light,
///             "dark" => ClientTheme::Dark,
///             _ => ClientTheme::Light, // defaults to light mode
///         }
///     }
/// }
/// ```
///
/// Assuming `client_preferences` is a valid [`ClientPreferences`]
/// ```ignore
/// let theme: ClientTheme = client_preferences.get("theme");
/// ```
pub struct ClientPreference {
    pub name: &'static str,
    pub value: String,
}

/// A request guard which stores the preferences sent from the client.
pub struct ClientPreferences(Vec<ClientPreference>);

impl ClientPreferences {
    /// Retrieve a particular preference which was sent to us from the client.
    ///
    /// `T` must implement `From<ClientPreference>`, which [`String`] already
    /// implements, in case the untouched cookie value is what needs to be handled.
    pub fn get<T: From<ClientPreference>>(self, name: &'static str) -> T {
        T::from(self.0.into_iter().find(|preference| preference.name == name).unwrap())
    }

    pub fn from_cookies(cookies: &CookieJar<'_>, preference_manager: &PreferenceManager) -> Self {
        let mut preferences: Vec<ClientPreference> = Vec::new();

        for preference in preference_manager.0.iter() {
            preferences.push(ClientPreference {
                name: preference.name,
                value: cookies
                    .get(&format!("preference-{}", preference.name))
                    .map(|cookie| cookie.value().to_string())
                    .unwrap_or(preference.default.to_string()),
            });
        }

        ClientPreferences(preferences)
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for ClientPreferences {
    type Error = CoreError;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let preference_manager = match request.rocket().state::<PreferenceManager>() {
            Some(preference_manager) => preference_manager,
            _ => return Outcome::Success(ClientPreferences(Vec::new())), // return an empty preferences vec if this instance doesnt support preferences
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
pub struct PreferenceManager(Vec<Preference>);

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
        self.0.push(Preference { name, default });

        self
    }
}

impl Default for PreferenceManager {
    fn default() -> Self {
        PreferenceManager(Vec::new())
    }
}

// type conversions
impl From<ClientPreference> for String {
    fn from(value: ClientPreference) -> Self {
        value.value
    }
}

impl From<ClientPreference> for &'static LanguageIdentifier {
    fn from(preference: ClientPreference) -> Self {
        get_locale(&preference.value)
    }
}
