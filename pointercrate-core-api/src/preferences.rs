use pointercrate_core::{error::CoreError, localization::get_locale};
use rocket::{
    http::CookieJar,
    request::{FromRequest, Outcome},
    Request,
};
use unic_langid::LanguageIdentifier;

// preference information from the client
pub struct ClientPreference {
    pub name: &'static str,
    pub value: String,
}

pub struct ClientPreferences(Vec<ClientPreference>);

impl ClientPreferences {
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

// static configurable preference manager for this pointercrate instance
pub struct Preference {
    pub name: &'static str,
    pub default: &'static str,
}

pub struct PreferenceManager(Vec<Preference>);

impl PreferenceManager {
    pub fn new() -> Self {
        PreferenceManager(Vec::new())
    }

    pub fn preference(mut self, name: &'static str, default: &'static str) -> Self {
        self.0.push(Preference { name, default });

        self
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
