use crate::{error::PointercrateError, model::user::User, permissions::Permissions, Result};
use futures::StreamExt;
use sqlx::{Error, PgConnection};

pub(super) struct FetchedUser {
    member_id: i32,
    name: String,
    permissions: i32, // FIXME(sqlx) once custom types are supported
    display_name: Option<String>,
    youtube_channel: Option<String>,
}

impl Into<User> for FetchedUser {
    fn into(self) -> User {
        User {
            id: self.member_id,
            name: self.name,
            permissions: Permissions::from_bits_truncate(self.permissions as u16),
            display_name: self.display_name,
            youtube_channel: self.youtube_channel,
        }
    }
}

impl User {
    pub async fn by_id(id: i32, connection: &mut PgConnection) -> Result<User> {
        let row = sqlx::query_as!(
            FetchedUser,
            "SELECT member_id, name, permissions::integer, display_name, youtube_channel::text FROM members WHERE member_id = $1",
            id
        )
        .fetch_one(connection)
        .await;

        match row {
            Err(Error::NotFound) =>
                Err(PointercrateError::ModelNotFound {
                    model: "User",
                    identified_by: id.to_string(),
                }),
            Err(err) => Err(err.into()),
            Ok(row) => Ok(row.into()),
        }
    }

    pub async fn by_name(name: &str, connection: &mut PgConnection) -> Result<User> {
        let row = sqlx::query_as!(
            FetchedUser,
            "SELECT member_id, name, permissions::integer, display_name, youtube_channel::text FROM members WHERE name = $1",
            name.to_string() // FIXME(sqlx)
        )
        .fetch_one(connection)
        .await;

        match row {
            Err(Error::NotFound) =>
                Err(PointercrateError::ModelNotFound {
                    model: "User",
                    identified_by: name.to_string(),
                }),
            Err(err) => Err(err.into()),
            Ok(row) => Ok(row.into()),
        }
    }

    /// Gets all users that have the given permission bits all set
    pub async fn by_permission(permissions: Permissions, connection: &mut PgConnection) -> Result<Vec<User>> {
        let mut stream = sqlx::query_as!(
            FetchedUser,
            "SELECT member_id, name, permissions::integer, display_name, youtube_channel::text FROM members WHERE permissions & \
             CAST($1::INTEGER AS BIT(16)) = CAST($1::INTEGER AS BIT(16))",
            permissions.bits() as i32
        )
        .fetch(connection);

        let mut users = Vec::new();

        while let Some(row) = stream.next().await {
            users.push(row?.into())
        }

        Ok(users)
    }
}
