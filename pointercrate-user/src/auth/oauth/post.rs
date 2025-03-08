use serde::Deserialize;


#[cfg(feature = "oauth2")]
#[derive(Debug, Deserialize)]
pub struct GoogleOauthPayload {
    credential: String,
}