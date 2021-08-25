use crate::{
    auth::{AuthenticatedUser, Claims},
    error::{Result},
    User,
};
use log::{debug, info};
use pointercrate_core::error::CoreError;
use sqlx::{Error, PgConnection};

impl AuthenticatedUser {
    pub async fn basic_auth(username: &str, password: &str, connection: &mut PgConnection) -> Result<AuthenticatedUser> {
        info!("We are expected to perform basic authentication");
        debug!("Trying to authorize user {}", username);

        Self::by_name(username, connection).await?.verify_password(password)
    }

    pub async fn token_auth(
        access_token: &str, csrf_token: Option<&str>, application_secret: &[u8], connection: &mut PgConnection,
    ) -> Result<AuthenticatedUser> {
        info!("We are expected to perform token authentication");

        // Well this is reassuring. Also we directly deconstruct it and only save the ID
        // so we don't accidentally use unsafe values later on
        let Claims { id, .. } = jsonwebtoken::dangerous_insecure_decode::<Claims>(&access_token)
            .map_err(|_| CoreError::Unauthorized)?
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
    }

    async fn by_id(id: i32, connection: &mut PgConnection) -> Result<AuthenticatedUser> {
        let row = sqlx::query!(
            r#"SELECT member_id, members.name, permissions::integer, display_name, youtube_channel::text, password_hash FROM members WHERE member_id = $1"#,
            id
        )
        .fetch_one(connection)
        .await;

        match row {
            Err(Error::RowNotFound) => Err(CoreError::Unauthorized.into()),
            Err(err) => Err(err.into()),
            Ok(row) =>
                Ok(AuthenticatedUser {
                    user: construct_from_row!(row),
                    password_hash: row.password_hash,
                }),
        }
    }

    async fn by_name(name: &str, connection: &mut PgConnection) -> Result<AuthenticatedUser> {
        let row = sqlx::query!(
            r#"SELECT member_id, members.name, permissions::integer, display_name, youtube_channel::text, password_hash FROM members WHERE members.name = $1"#,
            name.to_string()
        )
        .fetch_one(connection)
        .await;

        match row {
            Err(Error::RowNotFound) => Err(CoreError::Unauthorized.into()),
            Err(err) => Err(err.into()),
            Ok(row) =>
                Ok(AuthenticatedUser {
                    user: construct_from_row!(row),
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
