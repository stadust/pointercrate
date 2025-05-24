use chrono::{DateTime, Utc};
use jsonwebtoken::{Algorithm, DecodingKey, Validation};
use serde::Deserialize;

use crate::config;

#[derive(Debug, Deserialize)]
pub struct GoogleOauthPayload {
    pub credential: String,
}

#[derive(Deserialize)]
pub struct ValidatedGoogleCredentials {
    sub: String,
}

impl ValidatedGoogleCredentials {
    pub fn google_account_id(&self) -> &str {
        self.sub.as_ref()
    }
}

#[derive(Deserialize)]
pub struct SigningKey {
    e: String,
    n: String,
    kid: String,
    alg: Algorithm,
}

#[derive(Default, Deserialize)]
pub struct GoogleCertificateDatabase {
    pub keys: Vec<SigningKey>,

    #[serde(default)]
    pub expiry: Option<DateTime<Utc>>,
}

impl GoogleCertificateDatabase {
    pub fn needs_refresh(&self) -> bool {
        match self.expiry {
            None => true,
            Some(expiry) => Utc::now() >= expiry,
        }
    }

    pub fn validate_credentials(&self, creds: &str) -> Option<ValidatedGoogleCredentials> {
        let header = jsonwebtoken::decode_header(creds).ok()?;
        let key = self.keys.iter().find(|key| Some(key.kid.as_ref()) == header.kid.as_deref())?;

        let mut validation = Validation::new(key.alg);
        validation.set_issuer(&["accounts.google.com", "https://accounts.google.com"]);
        validation.set_audience(&[config::google_client_id()]);
        validation.required_spec_claims.extend(["iss".to_string(), "aud".to_string()]);

        jsonwebtoken::decode(creds, &DecodingKey::from_rsa_components(&key.n, &key.e).ok()?, &validation)
            .map(|data| data.claims)
            .inspect_err(|err| {
                use jsonwebtoken::errors::ErrorKind::*;

                match err.kind() {
                    // With these, we don't run into any danger of accidentally logging credentials
                    InvalidToken
                    | InvalidSignature
                    | InvalidEcdsaKey
                    | InvalidRsaKey(_)
                    | RsaFailedSigning
                    | InvalidAlgorithmName
                    | InvalidKeyFormat
                    | MissingRequiredClaim(_)
                    | ExpiredSignature
                    | InvalidIssuer
                    | InvalidAudience
                    | InvalidSubject
                    | ImmatureSignature
                    | InvalidAlgorithm
                    | MissingAlgorithm => log::warn!("Failure to validate credentials allegedly received from google: {:?}", err),
                    // All others, better be on the safe side and not log the actual error
                    _ => log::warn!("Failure to parse/validate credentials allegedly received from google"),
                }
            })
            .ok()
    }
}
