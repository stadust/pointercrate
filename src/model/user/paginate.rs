use crate::{
    error::PointercrateError,
    model::user::{get::FetchedUser, User},
    permissions::Permissions,
    util::{non_nullable, nullable},
    Result,
};
use futures::StreamExt;
use serde::Deserialize;
use sqlx::{PgConnection, Row};
use sqlx_core::postgres::PgRow;

#[derive(Deserialize, Debug, Clone)]
pub struct UserPagination {
    #[serde(rename = "before", default, deserialize_with = "non_nullable")]
    pub before_id: Option<i32>,

    #[serde(rename = "after", default, deserialize_with = "non_nullable")]
    pub after_id: Option<i32>,

    #[serde(default, deserialize_with = "non_nullable")]
    pub limit: Option<u8>,

    #[serde(default, deserialize_with = "non_nullable")]
    pub name: Option<String>,

    #[serde(default, deserialize_with = "nullable")]
    pub display_name: Option<Option<String>>,

    #[serde(default, deserialize_with = "non_nullable")]
    pub has_permissions: Option<Permissions>,
}

impl UserPagination {
    pub async fn page(&self, connection: &mut PgConnection) -> Result<Vec<User>> {
        if let Some(limit) = self.limit {
            if limit < 1 || limit > 100 {
                return Err(PointercrateError::InvalidPaginationLimit)
            }
        }

        if let (Some(after), Some(before)) = (self.before_id, self.after_id) {
            if after < before {
                return Err(PointercrateError::AfterSmallerBefore)
            }
        }

        let mut stream = sqlx::query_as(include_str!("../../../sql/paginate_users.sql"))
            .bind(self.before_id)
            .bind(self.after_id)
            .bind(self.name.as_ref())
            .bind(self.display_name.as_ref())
            .bind(self.display_name == Some(None))
            .bind(self.has_permissions.map(|p| p.bits() as i32))
            .bind(self.limit.unwrap_or(50) as i32)
            .fetch(connection);

        let mut users = Vec::new();

        while let Some(row) = stream.next().await {
            let row: PgRow = row?;

            let perms_as_i32: i32 = row.get("permissions");

            users.push(User {
                id: row.get("member_id"),
                name: row.get("name"),
                permissions: Permissions::from_bits_truncate(perms_as_i32 as u16),
                display_name: row.get("display_name"),
                youtube_channel: row.get("youtube_channel"),
            })
        }

        Ok(users)
    }
}
