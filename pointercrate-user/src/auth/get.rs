use std::collections::HashSet;

use crate::{
    auth::{AccessClaims, AuthenticatedUser},
    error::{Result, UserError},
    User,
};
use jsonwebtoken::{Algorithm, DecodingKey, Validation};
use log::{debug, info};
use pointercrate_core::{config, error::CoreError};
use serde::Deserialize;
use sqlx::{Error, PgConnection};

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
    pub async fn basic_auth(username: &str, password: &str, connection: &mut PgConnection) -> Result<AuthenticatedUser> {
        info!("We are expected to perform basic authentication");
        debug!("Trying to authorize user {}", username);

        Self::by_name(username, connection).await?.verify_password(password)
    }

    pub async fn token_auth(access_token: &str, csrf_token: Option<&str>, connection: &mut PgConnection) -> Result<AuthenticatedUser> {
        info!("We are expected to perform token authentication");

        // Well this is reassuring. Also we directly deconstruct it and only save the ID
        // so we don't accidentally use unsafe values later on
        let mut no_validation = Validation::default();
        no_validation.insecure_disable_signature_validation();
        no_validation.validate_exp = false;
        no_validation.required_spec_claims = HashSet::new();

        let AccessClaims { id, .. } = jsonwebtoken::decode(access_token, &DecodingKey::from_secret(b""), &no_validation)
            .map_err(|_| CoreError::Unauthorized)?
            .claims;

        debug!("The token identified the user with id {}, validating...", id);

        // Note that at this point we haven't validated the access token OR the csrf token yet.
        // However, the key they are signed with encompasses the password salt for the user they supposedly
        // identify, so we need to retrieve that.
        let user = Self::by_id(id, connection).await?.validate_access_token(access_token)?;

        if let Some(csrf_token) = csrf_token {
            user.validate_csrf_token(csrf_token)?
        }

        Ok(user)
    }

    pub async fn oauth2_callback(code: &str, existing_id: Option<i32>, connection: &mut PgConnection) -> Result<AuthenticatedUser> {
        info!("We are expected to perform Google OAuth2 authentication");

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
        validation.validate_nbf = false;
        validation.validate_aud = false;

        let user_info = jsonwebtoken::decode::<GoogleUserInfo>(&response.id_token, &key, &validation)
            .map_err(|_| CoreError::Unauthorized)?
            .claims;

        match User::by_google_account(&user_info.id, connection).await {
            Ok(user) => Ok(AuthenticatedUser {
                user,
                password_hash: None,
                google_account_id: Some(user_info.id),
                email_address: Some(user_info.email),
            }),
            Err(UserError::UserNotFoundGoogleAccount { .. }) => {
                if let Some(id) = existing_id {
                    let user = AuthenticatedUser::by_id(id, connection).await?;

                    if user.google_account_id.is_some() {
                        return Err(CoreError::Unauthorized.into());
                    }

                    let updated_user = sqlx::query!(
                        "UPDATE members SET google_account_id = $1, email_address = ($2::text)::email WHERE member_id = $3 RETURNING member_id",
                        user_info.id,
                        user_info.email,
                        id
                    )
                    .fetch_one(connection)
                    .await?;

                    if updated_user.member_id != id {
                        return Err(Error::RowNotFound.into());
                    }

                    Ok(AuthenticatedUser {
                        user: User { id, ..user.user },
                        password_hash: None,
                        google_account_id: Some(user_info.id),
                        email_address: Some(user_info.email),
                    })
                } else {
                    // This will never conflict with an existing user
                    // According to Google, the account ID is always unique
                    // https://developers.google.com/identity/openid-connect/openid-connect#an-id-tokens-payload
                    let name = format!("{}#{}", user_info.name, user_info.id);

                    let id = sqlx::query!(
                        "INSERT INTO
                            members (email_address, name, display_name, google_account_id)
                        VALUES
                            (($1::text)::email, $2, $3, $4) RETURNING member_id
                        ",
                        user_info.email,
                        name,
                        user_info.name,
                        user_info.id
                    )
                    .fetch_one(connection)
                    .await?
                    .member_id;

                    Ok(AuthenticatedUser {
                        user: User {
                            id,
                            name,
                            permissions: 0,
                            display_name: Some(user_info.name),
                            youtube_channel: None,
                        },
                        password_hash: None,
                        google_account_id: Some(user_info.id),
                        email_address: Some(user_info.email),
                    })
                }
            },
            Err(err) => Err(err),
        }
    }

    async fn by_id(id: i32, connection: &mut PgConnection) -> Result<AuthenticatedUser> {
        let row = sqlx::query!(
            r#"SELECT member_id, members.name, permissions::integer, display_name, youtube_channel::text, email_address::text, password_hash, google_account_id FROM members WHERE member_id = $1"#,
            id
        )
        .fetch_one(connection)
        .await;

        match row {
            Err(Error::RowNotFound) => Err(CoreError::Unauthorized.into()),
            Err(err) => Err(err.into()),
            Ok(row) => Ok(AuthenticatedUser {
                user: construct_from_row!(row),
                password_hash: row.password_hash,
                email_address: row.email_address,
                google_account_id: row.google_account_id,
            }),
        }
    }

    async fn by_name(name: &str, connection: &mut PgConnection) -> Result<AuthenticatedUser> {
        let row = sqlx::query!(
            r#"SELECT member_id, members.name, permissions::integer, display_name, youtube_channel::text, email_address::text, password_hash, google_account_id FROM members WHERE members.name = $1"#,
            name.to_string()
        )
        .fetch_one(connection)
        .await;

        match row {
            Err(Error::RowNotFound) => Err(CoreError::Unauthorized.into()),
            Err(err) => Err(err.into()),
            Ok(row) => Ok(AuthenticatedUser {
                user: construct_from_row!(row),
                password_hash: row.password_hash,
                email_address: row.email_address,
                google_account_id: row.google_account_id,
            }),
        }
    }
}
