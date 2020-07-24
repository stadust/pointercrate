use super::Submitter;
use crate::{error::PointercrateError, util::non_nullable, Result};
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use sqlx::{PgConnection, Row};

#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct SubmitterPagination {
    #[serde(rename = "before", default, deserialize_with = "non_nullable")]
    pub before_id: Option<i32>,

    #[serde(rename = "after", default, deserialize_with = "non_nullable")]
    pub after_id: Option<i32>,

    #[serde(default, deserialize_with = "non_nullable")]
    pub limit: Option<u8>,

    #[serde(default, deserialize_with = "non_nullable")]
    banned: Option<bool>,
}

impl SubmitterPagination {
    pub async fn page(&self, connection: &mut PgConnection) -> Result<Vec<Submitter>> {
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

        let query = if self.before_id.is_some() && self.after_id.is_none() {
            "SELECT submitter_id, banned FROM submitters WHERE (submitter_id < $1 OR $1 IS NULL) AND (submitter_id > $2 OR $2 IS NULL) AND \
             (banned = $3 OR $3 IS NULL) ORDER BY submitter_id DESC LIMIT $4 "
        } else {
            "SELECT submitter_id, banned FROM submitters WHERE (submitter_id < $1 OR $1 IS NULL) AND (submitter_id > $2 OR $2 IS NULL) AND \
             (banned = $3 OR $3 IS NULL) ORDER BY submitter_id ASC LIMIT $4 "
        };

        let mut stream = sqlx::query(query)
            .bind(self.before_id)
            .bind(self.after_id)
            .bind(self.banned)
            .bind(self.limit.unwrap_or(50) as i32 + 1)
            .fetch(connection);

        let mut submitters = Vec::new();

        while let Some(row) = stream.next().await {
            let row = row?;

            submitters.push(Submitter {
                id: row.get("submitter_id"),
                banned: row.get("banned"),
            })
        }

        Ok(submitters)
    }
}
