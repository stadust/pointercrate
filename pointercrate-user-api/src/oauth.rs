use std::sync::Arc;

use chrono::{Duration, Utc};
use pointercrate_user::auth::oauth::{GoogleCertificateDatabase, ValidatedGoogleCredentials};
use reqwest::Client;
use rocket::tokio::sync::RwLock;

pub const GOOGLE_CERT_ENDPOINT: &'static str = "https://www.googleapis.com/oauth2/v3/certs";

#[derive(Default)]
pub struct GoogleCertificateStore {
    db: Arc<RwLock<GoogleCertificateDatabase>>,
}

#[allow(dead_code)] /* fields exist specifically for Debug output */
#[derive(Debug)]
pub enum CertificateRefreshError {
    Reqwest(reqwest::Error),
    MalformedCacheControlHeader(String),
    MissingCacheControlHeader,
}

impl From<reqwest::Error> for CertificateRefreshError {
    fn from(value: reqwest::Error) -> Self {
        CertificateRefreshError::Reqwest(value)
    }
}

impl GoogleCertificateStore {
    pub async fn validate_credentials(&self, creds: &str) -> Option<ValidatedGoogleCredentials> {
        self.db.read().await.validate_credentials(creds)
    }

    pub async fn needs_refresh(&self) -> bool {
        self.db.read().await.needs_refresh()
    }

    pub async fn refresh(&self) -> Result<(), CertificateRefreshError> {
        let mut guard = self.db.write().await;

        // Between calling needs_refresh and us taking the write lock, did the need
        // for a refresh potentially go away (e.g. because some other task did the refresh in the meantime)?
        if !guard.needs_refresh() {
            return Ok(());
        }

        let client = Client::new();

        let request_time = Utc::now();
        let response = client.get(GOOGLE_CERT_ENDPOINT).send().await?;

        let cc_header = response
            .headers()
            .get("Cache-Control")
            .ok_or(CertificateRefreshError::MissingCacheControlHeader)?;
        let cc_header = cc_header
            .to_str()
            .map_err(|_| CertificateRefreshError::MalformedCacheControlHeader("unparsable".to_string()))?;

        let max_age_directive = cc_header
            .split(',')
            .find(|directive| directive.trim().starts_with("max-age"))
            .ok_or_else(|| CertificateRefreshError::MalformedCacheControlHeader(cc_header.to_string()))?;

        let mut parts = max_age_directive.split('=');
        _ = parts.next(); /* Split<'_, T> implements FusedIterator, so can ignore None handling here */
        let age_str = parts
            .next()
            .ok_or_else(|| CertificateRefreshError::MalformedCacheControlHeader(max_age_directive.to_string()))?;
        let age = age_str
            .parse()
            .map_err(|_| CertificateRefreshError::MalformedCacheControlHeader(age_str.to_string()))?;

        let expiry = request_time + Duration::seconds(age);

        let mut new_db = response.json::<GoogleCertificateDatabase>().await?;
        new_db.expiry = Some(expiry);

        *guard = new_db;

        Ok(())
    }
}
