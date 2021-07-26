use crate::{error::PointercrateError, model::user::User, permissions::Permissions, Result};
use futures::StreamExt;
use sqlx::{Error, PgConnection};

macro_rules! query_user {
    ($connection: expr, $query:expr, $id: expr, $($param: expr),*) => {{
        let row = sqlx::query!($query, $id, $($param),*).fetch_one($connection).await;

        match row {
            Err(Error::RowNotFound) =>
                Err(PointercrateError::ModelNotFound {
                    model: "User",
                    identified_by: $id.to_string(),
                }),
            Err(err) => Err(err.into()),
            Ok(row) => Ok(User {
                id: row.member_id,
                name: row.name,
                permissions: Permissions::from_bits_truncate(row.permissions.unwrap() as u16),
                display_name: row.display_name,
                youtube_channel: row.youtube_channel,
            }),
        }
    }};
}

impl User {
    pub async fn by_id(id: i32, connection: &mut PgConnection) -> Result<User> {
        query_user!(
            connection,
            "SELECT member_id, name, permissions::integer, display_name, youtube_channel::text FROM members WHERE member_id = $1",
            id,
        )
    }

    pub async fn by_name(name: &str, connection: &mut PgConnection) -> Result<User> {
        query_user!(
            connection,
            "SELECT member_id, name, CAST(permissions AS integer), display_name, youtube_channel::text FROM members WHERE name = $1",
            name,
        )
    }

    /// Gets all users that have the given permission bits all set
    pub async fn by_permission(permissions: Permissions, connection: &mut PgConnection) -> Result<Vec<User>> {
        let mut stream = sqlx::query!(
            "SELECT member_id, name, permissions::integer, display_name, youtube_channel::text FROM members WHERE permissions & \
             CAST($1::INTEGER AS BIT(16)) = CAST($1::INTEGER AS BIT(16))",
            permissions.bits() as i32
        )
        .fetch(connection);

        let mut users = Vec::new();

        while let Some(row) = stream.next().await {
            let row = row?;

            users.push(User {
                id: row.member_id,
                name: row.name,
                permissions: Permissions::from_bits_truncate(row.permissions.unwrap() as u16),
                display_name: row.display_name,
                youtube_channel: row.youtube_channel,
            })
        }

        Ok(users)
    }
}
