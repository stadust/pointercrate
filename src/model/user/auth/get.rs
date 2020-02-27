use crate::{
    error::PointercrateError,
    model::user::{
        auth::{patch::PatchMe, AuthenticatedUser, Claims},
        User,
    },
    permissions::Permissions,
    Result,
};
use log::{debug, info, warn};
use sqlx::{Error, PgConnection};

struct FetchedUser {
    member_id: i32,
    name: String,
    permissions: i32, // FIXME(sqlx) once custom types are supported
    display_name: Option<String>,
    youtube_channel: Option<String>,
    password_hash: String,
}

/// Enum representing a parsed `Authorization` header
#[derive(Debug)]
pub enum Authorization {
    /// No `Authorization` header has been provided
    Unauthorized,

    /// The chosen authorization method was `Basic`
    Basic { username: String, password: String },

    /// The chosen authorization method was `Bearer`
    Token { access_token: String, csrf_token: Option<String> },
}

impl AuthenticatedUser {
    pub async fn invalidate_all_tokens(authorization: Authorization, connection: &mut PgConnection) -> Result<()> {
        let user = Self::basic_auth(&authorization, connection).await?;

        if let Authorization::Basic { password, .. } = authorization {
            let patch = PatchMe {
                password: Some(password),
                display_name: None,
                youtube_channel: None,
            };

            warn!("Invalidating all access tokens for user {}", user.inner());

            user.apply_patch(patch, connection).await?;

            Ok(())
        } else {
            // actually unreachable
            Err(PointercrateError::Unauthorized)
        }
    }

    pub async fn basic_auth(auth: &Authorization, connection: &mut PgConnection) -> Result<AuthenticatedUser> {
        info!("We are expected to perform basic authentication");

        // TODO: ratelimiting here (what did I mean by this?)

        if let Authorization::Basic { username, password } = auth {
            debug!("Trying to authorize user {}", username);

            Self::by_name(username, connection).await?.verify_password(password)
        } else {
            warn!("No basic authentication found");

            Err(PointercrateError::Unauthorized)
        }
    }

    pub async fn token_auth(auth: &Authorization, application_secret: &[u8], connection: &mut PgConnection) -> Result<AuthenticatedUser> {
        info!("We are expected to perform token authentication");

        if let Authorization::Token { access_token, csrf_token } = auth {
            // Well this is reassuring. Also we directly deconstruct it and only save the ID
            // so we don't accidentally use unsafe values later on
            let Claims { id, .. } = jsonwebtoken::dangerous_unsafe_decode::<Claims>(&access_token)
                .map_err(|_| PointercrateError::Unauthorized)?
                .claims;

            debug!("The token identified the user with id {}, validating...", id);

            // Note that at this point we haven't validated the access token OR the csrf token yet.
            // However, the key they are signed with encompasses the password salt for the user they supposedly
            // identify, so we need to retrieve that.
            let user = Self::by_id(id, connection)
                .await?
                .validate_token(&access_token, application_secret)?;

            if let Some(ref csrf_token) = csrf_token {
                user.validate_csrf_token(csrf_token, application_secret)?
            }

            Ok(user)
        } else {
            warn!("No token authentication found");

            Err(PointercrateError::Unauthorized)
        }
    }

    async fn by_id(id: i32, connection: &mut PgConnection) -> Result<AuthenticatedUser> {
        let row = sqlx::query_as!(
            FetchedUser,
            "SELECT member_id, name, permissions::integer, display_name, youtube_channel::text, password_hash FROM members WHERE \
             member_id = $1",
            id
        )
        .fetch_one(connection)
        .await;

        match row {
            Err(Error::NotFound) => Err(PointercrateError::Unauthorized),
            Err(err) => Err(err.into()),
            Ok(row) =>
                Ok(AuthenticatedUser {
                    user: User {
                        id,
                        name: row.name,
                        permissions: Permissions::from_bits_truncate(row.permissions as u16),
                        display_name: row.display_name,
                        youtube_channel: row.youtube_channel,
                    },
                    password_hash: row.password_hash,
                }),
        }
    }

    async fn by_name(name: &str, connection: &mut PgConnection) -> Result<AuthenticatedUser> {
        let row = sqlx::query_as!(
            FetchedUser,
            "SELECT member_id, name, permissions::integer, display_name, youtube_channel::text, password_hash FROM members WHERE name = $1",
            name.to_string()
        )
        .fetch_one(connection)
        .await;

        match row {
            Err(Error::NotFound) => Err(PointercrateError::Unauthorized),
            Err(err) => Err(err.into()),
            Ok(row) =>
                Ok(AuthenticatedUser {
                    user: User {
                        id: row.member_id,
                        name: row.name,
                        permissions: Permissions::from_bits_truncate(row.permissions as u16),
                        display_name: row.display_name,
                        youtube_channel: row.youtube_channel,
                    },
                    password_hash: row.password_hash,
                }),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        error::PointercrateError,
        model::user::{AuthenticatedUser, Authorization},
    };

    #[actix_rt::test]
    async fn test_successful_basic_auth() {
        let mut connection = crate::test::test_setup().await;

        let result = AuthenticatedUser::basic_auth(
            &Authorization::Basic {
                username: "stadust_existing".to_owned(),
                password: "password1234567890".to_string(),
            },
            &mut connection,
        )
        .await;

        assert!(result.is_ok(), "{:?}", result.err().unwrap());
        assert_eq!(result.unwrap().inner().name, "stadust_existing");
    }

    #[actix_rt::test]
    async fn test_basic_auth_fail_invalid_name() {
        let mut connection = crate::test::test_setup().await;

        let result = AuthenticatedUser::basic_auth(
            &Authorization::Basic {
                username: "stadust_nonexisting".to_owned(),
                password: "password1234567890".to_string(),
            },
            &mut connection,
        )
        .await;

        assert!(result.is_err(), "{:?}", result.ok().unwrap().inner());
        assert_eq!(result.err().unwrap(), PointercrateError::Unauthorized);
    }

    #[actix_rt::test]
    async fn test_basic_auth_fail_invalid_password() {
        let mut connection = crate::test::test_setup().await;

        let result = AuthenticatedUser::basic_auth(
            &Authorization::Basic {
                username: "stadust_existing".to_owned(),
                password: "wrong password".to_string(),
            },
            &mut connection,
        )
        .await;

        assert!(result.is_err(), "{:?}", result.ok().unwrap().inner());
        assert_eq!(result.err().unwrap(), PointercrateError::Unauthorized);
    }
}
