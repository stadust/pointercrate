use crate::{
    error::PointercrateError,
    permissions::Permissions,
    util::{non_nullable, nullable},
    Result,
};
use futures::StreamExt;
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
    pub has_permissions: Option<Permissions>,

    #[serde(default, deserialize_with = "non_nullable")]
    pub any_permissions: Option<Permissions>,
}

/// Model representing a user in the database
#[derive(Debug, Serialize, Hash, Eq, PartialEq)]
pub struct ListedUser {
    /// The [`User`]'s unique ID. This is used to identify users and cannot be changed.
    pub id: i32,

    /// The [`User`]'s unique username. This is used to log-in and cannot be changed.
    pub name: String,

    pub permissions: Permissions,

    /// A user-customizable name for each [`User`].
    ///
    /// If set to anything other than [`None`], the value set here will be displayed everywhere the
    /// username would be displayed otherwise. This value is not guaranteed to be unique and
    /// cannot be used to identify a user. In particular, this value cannot be used for log-in
    pub display_name: Option<String>,

    /// A user-customizable link to a [YouTube](https://youtube.com) channel
    pub youtube_channel: Option<String>,
}

impl UserPagination {
    pub async fn page(&self, connection: &mut PgConnection) -> Result<Vec<ListedUser>> {
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

        let order = if self.after_id.is_none() && self.before_id.is_some() {
            "DESC"
        } else {
            "ASC"
        };

        let query = format!(include_str!("../../../sql/paginate_users.sql"), order);

        let mut stream = sqlx::query(&query)
            .bind(self.before_id)
            .bind(self.after_id)
            .bind(self.name.as_ref())
            .bind(self.display_name.as_ref())
            .bind(self.display_name == Some(None))
            .bind(self.has_permissions.map(|p| p.bits() as i32))
            .bind(self.any_permissions.map(|p| p.bits() as i32))
            .bind(self.name_contains.as_ref())
            .bind(self.limit.unwrap_or(50) as i32 + 1)
            .fetch(connection);

        let mut users = Vec::new();

        while let Some(row) = stream.next().await {
            let row: PgRow = row?;

            let perms_as_i32: i32 = row.get("permissions");

            users.push(ListedUser {
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

impl ListedUser {
    pub fn name(&self) -> &str {
        match self.display_name {
            Some(ref name) => name,
            None => self.name.as_ref(),
        }
    }

    /// Gets all users that have the given permission bits all set
    pub async fn by_permission(permissions: Permissions, connection: &mut PgConnection) -> Result<Vec<ListedUser>> {
        let mut stream = sqlx::query!(
            "SELECT member_id, name, permissions::integer, display_name, youtube_channel::text FROM members WHERE permissions & \
             CAST($1::INTEGER AS BIT(16)) = CAST($1::INTEGER AS BIT(16))",
            permissions.bits() as i32
        )
        .fetch(connection);

        let mut users = Vec::new();

        while let Some(row) = stream.next().await {
            let row = row?;

            users.push(ListedUser {
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
