use crate::{model::user::User, permissions::Permissions, Result};
use sqlx::PgConnection;

struct FetchedUser {
    member_id: i32,
    name: String,
    permissions: i32, // FIXME(sqlx) once custom types are supported
    display_name: Option<String>,
    youtube_channel: Option<String>,
}

impl User {
    pub async fn by_id(id: i32, connection: &mut PgConnection) -> Result<User> {
        let row = sqlx::query_as!(
            FetchedUser,
            "SELECT member_id, name, permissions::integer, display_name, youtube_channel::text FROM members WHERE member_id = $1",
            id
        )
        .fetch_one(connection)
        .await?;

        Ok(User {
            id,
            name: row.name,
            permissions: Permissions::from_bits_truncate(row.permissions as u16),
            display_name: row.display_name,
            youtube_channel: row.youtube_channel,
        })
    }

    pub async fn by_name(name: &str, connection: &mut PgConnection) -> Result<User> {
        let row = sqlx::query_as!(
            FetchedUser,
            "SELECT member_id, name, permissions::integer, display_name, youtube_channel::text FROM members WHERE name = $1",
            name.to_string() // FIXME(sqlx)
        )
        .fetch_one(connection)
        .await?;

        Ok(User {
            id: row.member_id,
            name: row.name,
            permissions: Permissions::from_bits_truncate(row.permissions as u16),
            display_name: row.display_name,
            youtube_channel: row.youtube_channel,
        })
    }
}
