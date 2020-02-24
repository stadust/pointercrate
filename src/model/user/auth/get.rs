use crate::{
    model::user::{auth::AuthenticatedUser, User},
    permissions::Permissions,
    Result,
};
use sqlx::PgConnection;

struct FetchedUser {
    member_id: i32,
    name: String,
    permissions: i32, // FIXME(sqlx) once custom types are supported
    display_name: Option<String>,
    youtube_channel: Option<String>,
    password_hash: String,
}

impl AuthenticatedUser {
    pub async fn by_id(id: i32, connection: &mut PgConnection) -> Result<AuthenticatedUser> {
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
}
