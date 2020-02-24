use crate::{
    cistring::CiStr,
    error::PointercrateError,
    model::user::{
        auth::{patch::PatchMe, AuthenticatedUser, Claims},
        User,
    },
    permissions::Permissions,
    Result,
};
use log::{debug, info, trace, warn};
use sqlx::PgConnection;

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
        let mut user = Self::basic_auth(&authorization, connection).await?;

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

    /// Prepares this connection such that all audit log entries generated while using it are
    /// attributed to
    async fn audit_connection(&self, connection: &mut PgConnection) -> Result<()> {
        trace!(
            "Creating connection of which usage will be attributed to user {} in audit logs",
            self.user.id
        );

        sqlx::query!("CREATE TEMPORARY TABLE IF NOT EXISTS active_user (id INTEGER)")
            .execute(connection)
            .await?;
        sqlx::query!("DELETE FROM active_user").execute(connection).await?;
        sqlx::query!("INSERT INTO active_user (id) VALUES ($1)", self.user.id)
            .execute(connection)
            .await?;

        Ok(())
    }

    pub async fn basic_auth(auth: &Authorization, connection: &mut PgConnection) -> Result<AuthenticatedUser> {
        info!("We are expected to perform basic authentication");

        // TODO: ratelimiting here (what did I mean by this?)

        if let Authorization::Basic { username, password } = auth {
            debug!("Trying to authorize user {}", username);

            let user = Self::by_name(username, connection).await?.verify_password(password)?;

            user.audit_connection(connection).await?;

            Ok(user)
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

            user.audit_connection(connection).await?;

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
        .await?;

        Ok(AuthenticatedUser {
            user: User {
                id,
                name: row.name,
                permissions: Permissions::from_bits_truncate(row.permissions as u16),
                display_name: row.display_name,
                youtube_channel: row.youtube_channel,
            },
            password_hash: row.password_hash,
        })
    }

    async fn by_name(name: &str, connection: &mut PgConnection) -> Result<AuthenticatedUser> {
        let row = sqlx::query_as!(
            FetchedUser,
            "SELECT member_id, name, permissions::integer, display_name, youtube_channel::text, password_hash FROM members WHERE name = $1",
            name.to_string()
        )
        .fetch_one(connection)
        .await?;

        Ok(AuthenticatedUser {
            user: User {
                id: row.member_id,
                name: row.name,
                permissions: Permissions::from_bits_truncate(row.permissions as u16),
                display_name: row.display_name,
                youtube_channel: row.youtube_channel,
            },
            password_hash: row.password_hash,
        })
    }
}
