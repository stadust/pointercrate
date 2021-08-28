use crate::{error::Result, User};
use futures::StreamExt;
use pointercrate_core::{
    error::CoreError,
    permission::Permission,
    util::{non_nullable, nullable},
};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgRow, PgConnection, Row};

#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct UserPagination {
    #[serde(rename = "before", default, deserialize_with = "non_nullable")]
    pub before_id: Option<i32>,

    #[serde(rename = "after", default, deserialize_with = "non_nullable")]
    pub after_id: Option<i32>,

    #[serde(default, deserialize_with = "non_nullable")]
    pub limit: Option<u8>,

    #[serde(default, deserialize_with = "non_nullable")]
    pub name: Option<String>,

    #[serde(default, deserialize_with = "non_nullable")]
    pub name_contains: Option<String>,

    #[serde(default, deserialize_with = "nullable")]
    pub display_name: Option<Option<String>>,

    #[serde(default, deserialize_with = "non_nullable")]
    pub has_permissions: Option<u16>,

    #[serde(default, deserialize_with = "non_nullable")]
    pub any_permissions: Option<u16>,
}

impl UserPagination {
    pub async fn page(&self, connection: &mut PgConnection) -> Result<Vec<User>> {
        if let Some(limit) = self.limit {
            if limit < 1 || limit > 100 {
                return Err(CoreError::InvalidPaginationLimit.into())
            }
        }

        if let (Some(after), Some(before)) = (self.before_id, self.after_id) {
            if after < before {
                return Err(CoreError::AfterSmallerBefore.into())
            }
        }

        let order = if self.after_id.is_none() && self.before_id.is_some() {
            "DESC"
        } else {
            "ASC"
        };

        let query = format!(include_str!("../sql/paginate_users.sql"), order);

        let mut stream = sqlx::query(&query)
            .bind(self.before_id)
            .bind(self.after_id)
            .bind(self.name.as_ref())
            .bind(self.display_name.as_ref())
            .bind(self.display_name == Some(None))
            .bind(self.has_permissions.map(|p| p as i32))
            .bind(self.any_permissions.map(|p| p as i32))
            .bind(self.name_contains.as_ref())
            .bind(self.limit.unwrap_or(50) as i32 + 1)
            .fetch(connection);

        let mut users = Vec::new();

        while let Some(row) = stream.next().await {
            let row: PgRow = row?;

            let perms_as_i32: i32 = row.get("permissions");

            users.push(User {
                id: row.get("member_id"),
                name: row.get("name"),
                permissions: perms_as_i32 as u16,
                display_name: row.get("display_name"),
                youtube_channel: row.get("youtube_channel"),
            })
        }

        Ok(users)
    }
}

impl User {
    pub async fn by_permission(permission: Permission, connection: &mut PgConnection) -> Result<Vec<User>> {
        User::by_permissions(permission.bit(), connection).await
    }

    /// Gets all users that have the given permission bits all set
    pub async fn by_permissions(permissions: u16, connection: &mut PgConnection) -> Result<Vec<User>> {
        let mut stream = sqlx::query!(
            "SELECT member_id, name, permissions::integer, display_name, youtube_channel::text FROM members WHERE permissions & \
             CAST($1::INTEGER AS BIT(16)) = CAST($1::INTEGER AS BIT(16))",
            permissions as i32
        )
        .fetch(connection);

        let mut users = Vec::new();

        while let Some(row) = stream.next().await {
            let row = row?;

            users.push(User {
                id: row.member_id,
                name: row.name,
                permissions: row.permissions.unwrap() as u16,
                display_name: row.display_name,
                youtube_channel: row.youtube_channel,
            })
        }

        Ok(users)
    }
}
