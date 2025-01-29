use jsonwebtoken::{Algorithm, DecodingKey, Validation};
use log::info;
use pointercrate_core::error::CoreError;
use serde::Deserialize;
use sqlx::{Error, PgConnection};

use crate::auth::AuthenticatedUser;
use crate::error::UserError;
use crate::{Result, User};

#[derive(Deserialize)]
struct GoogleTokenResponse {
    pub id_token: String,
}

#[derive(Deserialize)]
struct GoogleUserInfo {
    #[serde(rename = "sub")]
    pub id: String,
    pub email: String,
    pub name: String,
}

impl AuthenticatedUser {
    pub async fn oauth2_callback(credential: &str, connection: &mut PgConnection) -> Result<AuthenticatedUser> {
        info!("We are expected to perform Google OAuth2 authentication");

        // TODO: verify the token
        let key = DecodingKey::from_secret(&[]);
        let mut validation = Validation::new(Algorithm::HS256);
        validation.insecure_disable_signature_validation();
        validation.validate_exp = false;
        validation.validate_nbf = false;
        validation.validate_aud = false;

        let user_info = jsonwebtoken::decode::<GoogleUserInfo>(credential, &key, &validation)
            .map_err(|_| CoreError::Unauthorized)?
            .claims;

        match User::by_google_account(&user_info.id, connection).await {
            Ok(user) => Ok(AuthenticatedUser::oauth2(user, user_info.email, user_info.id)),
            Err(UserError::UserNotFoundGoogleAccount { .. }) => {
                // This will never conflict with an existing user
                // According to Google, the account ID is always unique
                // https://developers.google.com/identity/openid-connect/openid-connect#an-id-tokens-payload

                let id = sqlx::query!(
                    "INSERT INTO
                    members (email_address, name, display_name, google_account_id)
                VALUES
                    (($1::text)::email, $2, $3, $4) RETURNING member_id
                ",
                    user_info.email,
                    user_info.id,
                    user_info.id,
                    user_info.id
                )
                .fetch_one(connection)
                .await?
                .member_id;

                Ok(Self::oauth2(
                    User {
                        id,
                        name: user_info.id.clone(),
                        permissions: 0,
                        display_name: Some(user_info.id.clone()),
                        youtube_channel: None,
                    },
                    user_info.id,
                    user_info.email,
                ))
            },
            Err(err) => Err(err),
        }
    }
}
