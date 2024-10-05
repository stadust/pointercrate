use super::OAuth2AuthenticatedUser;
use crate::auth::oauth2::GoogleUserInfo;
use crate::auth::AuthenticatedUser;
use crate::config;
use crate::error::UserError;
use crate::{Result, User};
use jsonwebtoken::{Algorithm, DecodingKey, Validation};
use pointercrate_core::error::CoreError;
use serde::Deserialize;
use sqlx::{Error, PgConnection};

impl AuthenticatedUser {
    /// Resolves the given google oauth code for a google access token. Then tries
    /// to find a pointercrate account linked to the google account for which we now have
    /// an access token. If it finds one, return it. Otherwise, create a new pointercrate
    /// account and link it to the google account.
    pub async fn by_oauth_code(code: &str, mut connection: &mut PgConnection) -> Result<Self> {
        let guser_info = get_oauth2_id(code).await?;

        match OAuth2AuthenticatedUser::by_google_account(&guser_info.id, &mut connection).await {
            Ok(oauth_user) => Ok(AuthenticatedUser::OAuth2(oauth_user)),
            Err(UserError::UserNotFoundGoogleAccount { .. }) => AuthenticatedUser::register_oauth(guser_info, &mut connection).await,
            Err(err) => Err(err.into()),
        }
    }
}

impl OAuth2AuthenticatedUser {
    async fn by_google_account(id: &str, connection: &mut PgConnection) -> Result<Self> {
        let row = sqlx::query!(
            r#"SELECT member_id, members.name, CAST(permissions AS integer), display_name, youtube_channel::text, password_hash FROM members WHERE google_account_id = $1"#,
            id
        )
        .fetch_one(connection)
        .await;

        match row {
            Err(Error::RowNotFound) => Err(UserError::UserNotFoundGoogleAccount {
                google_account_id: id.to_string(),
            }),
            Err(err) => Err(err.into()),
            Ok(row) => Ok(OAuth2AuthenticatedUser {
                user: construct_from_row!(row),
                b64_salt: row.password_hash,
                google_account_id: id.to_string(),
            }),
        }
    }
}

#[derive(Deserialize)]
struct GoogleTokenResponse {
    id_token: String,
}

pub(in crate::auth::oauth2) async fn get_oauth2_id(code: &str) -> Result<GoogleUserInfo> {
    let client = reqwest::Client::new();

    let response = client
        .post("https://oauth2.googleapis.com/token")
        .form(&[
            ("code", code),
            ("client_id", &config::google_client_id()),
            ("client_secret", &config::google_client_secret()),
            ("redirect_uri", &config::google_redirect_uri()),
            ("grant_type", "authorization_code"),
        ])
        .send()
        .await
        .map_err(|_| CoreError::Unauthorized)?;

    let response: GoogleTokenResponse = response.json().await.map_err(|_| CoreError::Unauthorized)?;

    // We can safely disable all validation here, as Google recommends to not
    // validate a fresh token, as it is guaranteed to be valid.
    //
    // https://developers.google.com/identity/openid-connect/openid-connect#obtainuserinfo
    let key = DecodingKey::from_secret(&[]);
    let mut validation = Validation::new(Algorithm::HS256);
    validation.insecure_disable_signature_validation();
    validation.validate_exp = false;
    validation.validate_aud = false;

    jsonwebtoken::decode::<GoogleUserInfo>(&response.id_token, &key, &validation)
        .map_err(|_| CoreError::Unauthorized.into())
        .map(|token_data| token_data.claims)
}
