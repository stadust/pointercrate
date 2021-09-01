use crate::error::Result;
use futures::StreamExt;
use pointercrate_core::{
    audit::NamedId,
    error::CoreError,
    util::{non_nullable, nullable},
};
use serde::{Deserialize, Serialize};
use sqlx::{PgConnection, Row};

#[derive(Deserialize, Serialize, Debug)]
pub struct PlayerClaimPagination {
    #[serde(default, deserialize_with = "non_nullable")]
    #[serde(rename = "before")]
    pub before_id: Option<i32>,

    #[serde(default, deserialize_with = "non_nullable")]
    #[serde(rename = "after")]
    pub after_id: Option<i32>,

    #[serde(default, deserialize_with = "non_nullable")]
    pub limit: Option<u8>,

    #[serde(default, deserialize_with = "non_nullable")]
    any_name_contains: Option<String>,

    #[serde(default, deserialize_with = "non_nullable")]
    verified: Option<bool>,
}

#[derive(Serialize)]
pub struct ListedClaim {
    #[serde(skip)]
    pub id: i32,
    user: NamedId,
    player: NamedId,
    verified: bool,
}

impl ListedClaim {
    pub async fn extremal_ids(connection: &mut PgConnection) -> Result<(i32, i32)> {
        let row = sqlx::query!(r#"SELECT MAX(id) AS "max_id!: i32", MIN(id) AS "min_id!: i32" FROM player_claims"#)
            .fetch_one(connection)
            .await?; // FIXME: crashes on empty table
        Ok((row.max_id, row.min_id))
    }
}

impl PlayerClaimPagination {
    pub async fn page(&self, connection: &mut PgConnection) -> Result<Vec<ListedClaim>> {
        if let Some(limit) = self.limit {
            if limit < 1 || limit > 100 {
                Err(CoreError::InvalidPaginationLimit)?
            }
        }

        if let (Some(after), Some(before)) = (self.before_id, self.after_id) {
            if after < before {
                Err(CoreError::AfterSmallerBefore)?
            }
        }

        let limit = self.limit.unwrap_or(50) as i32;

        let order = if self.after_id.is_none() && self.before_id.is_some() {
            "DESC"
        } else {
            "ASC"
        };

        let query = format!(include_str!("../../../sql/paginate_claims.sql"), order);

        let mut stream = sqlx::query(&query)
            .bind(self.before_id)
            .bind(self.after_id)
            .bind(self.any_name_contains.as_ref())
            .bind(self.verified)
            .bind(self.limit.unwrap_or(50) as i32 + 1)
            .fetch(connection);

        let mut claims = Vec::new();

        while let Some(row) = stream.next().await {
            let row = row?;

            claims.push(ListedClaim {
                id: row.get("id"),
                user: NamedId {
                    id: row.get("mid"),
                    name: Some(row.get("mname")),
                },
                player: NamedId {
                    id: row.get("pid"),
                    name: Some(row.get("pname")),
                },
                verified: row.get("verified"),
            })
        }

        Ok(claims)
    }
}
