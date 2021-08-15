use crate::{
    demon::{Demon, MinimalDemon},
    error::Result,
    player::DatabasePlayer,
};
use futures::stream::StreamExt;
use pointercrate_core::{error::CoreError, util::non_nullable};
use serde::{Deserialize, Serialize};
use sqlx::{PgConnection, Row};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DemonIdPagination {
    #[serde(default, deserialize_with = "non_nullable")]
    #[serde(rename = "before")]
    pub before_id: Option<i32>,

    #[serde(default, deserialize_with = "non_nullable")]
    #[serde(rename = "after")]
    pub after_id: Option<i32>,

    #[serde(default, deserialize_with = "non_nullable")]
    pub limit: Option<u8>,

    #[serde(default, deserialize_with = "non_nullable")]
    name: Option<String>,

    #[serde(default, deserialize_with = "non_nullable")]
    name_contains: Option<String>,

    #[serde(default, deserialize_with = "non_nullable")]
    requirement: Option<i16>,

    #[serde(default, deserialize_with = "non_nullable")]
    verifier_id: Option<i32>,
    #[serde(default, deserialize_with = "non_nullable")]
    publisher_id: Option<i32>,

    #[serde(default, deserialize_with = "non_nullable")]
    verifier_name: Option<String>,
    #[serde(default, deserialize_with = "non_nullable")]
    publisher_name: Option<String>,

    #[serde(default, deserialize_with = "non_nullable")]
    #[serde(rename = "requirement__gt")]
    requirement_gt: Option<i16>,

    #[serde(default, deserialize_with = "non_nullable")]
    #[serde(rename = "requirement__lt")]
    requirement_lt: Option<i16>,
}

impl DemonIdPagination {
    pub async fn page(&self, connection: &mut PgConnection) -> Result<Vec<Demon>> {
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

        let order = if self.after_id.is_none() && self.before_id.is_some() {
            "DESC"
        } else {
            "ASC"
        };

        let query = format!(include_str!("../../sql/paginate_demons_by_id.sql"), order);

        // FIXME(sqlx) once CITEXT is supported
        let mut stream = sqlx::query(&query)
            .bind(self.before_id)
            .bind(self.after_id)
            .bind(self.name.as_ref().map(|s| s.as_str()))
            .bind(self.requirement)
            .bind(self.requirement_lt)
            .bind(self.requirement_gt)
            .bind(self.verifier_id)
            .bind(self.verifier_name.as_ref().map(|s| s.as_str()))
            .bind(self.publisher_id)
            .bind(self.publisher_name.as_ref().map(|s| s.as_str()))
            .bind(self.name_contains.as_ref().map(|s| s.as_str()))
            .bind(self.limit.unwrap_or(50) as i32 + 1)
            .fetch(connection);

        let mut demons = Vec::new();

        while let Some(row) = stream.next().await {
            let row = row?;

            let video: Option<String> = row.get("video");

            demons.push(Demon {
                base: MinimalDemon {
                    id: row.get("demon_id"),
                    name: row.get("demon_name"),
                    position: row.get("position"),
                },
                requirement: row.get("requirement"),
                video,
                publisher: DatabasePlayer {
                    id: row.get("publisher_id"),
                    name: row.get("publisher_name"),
                    banned: row.get("publisher_banned"),
                },
                verifier: DatabasePlayer {
                    id: row.get("verifier_id"),
                    name: row.get("verifier_name"),
                    banned: row.get("verifier_banned"),
                },
                level_id: row.get::<Option<i64>, _>("level_id").map(|id| id as u64),
            })
        }

        Ok(demons)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DemonPositionPagination {
    #[serde(default, deserialize_with = "non_nullable")]
    #[serde(rename = "before")]
    pub before_position: Option<i16>,

    #[serde(default, deserialize_with = "non_nullable")]
    #[serde(rename = "after")]
    pub after_position: Option<i16>,

    #[serde(default, deserialize_with = "non_nullable")]
    pub limit: Option<u8>,

    #[serde(default, deserialize_with = "non_nullable")]
    name: Option<String>,

    #[serde(default, deserialize_with = "non_nullable")]
    name_contains: Option<String>,

    #[serde(default, deserialize_with = "non_nullable")]
    requirement: Option<i16>,

    #[serde(default, deserialize_with = "non_nullable")]
    verifier_id: Option<i32>,
    #[serde(default, deserialize_with = "non_nullable")]
    publisher_id: Option<i32>,

    #[serde(default, deserialize_with = "non_nullable")]
    verifier_name: Option<String>,
    #[serde(default, deserialize_with = "non_nullable")]
    publisher_name: Option<String>,

    #[serde(default, deserialize_with = "non_nullable")]
    #[serde(rename = "requirement__gt")]
    requirement_gt: Option<i16>,

    #[serde(default, deserialize_with = "non_nullable")]
    #[serde(rename = "requirement__lt")]
    requirement_lt: Option<i16>,
}

impl DemonPositionPagination {
    pub async fn page(&self, connection: &mut PgConnection) -> Result<Vec<Demon>> {
        if let Some(limit) = self.limit {
            if limit < 1 || limit > 100 {
                Err(CoreError::InvalidPaginationLimit)?
            }
        }

        if let (Some(after), Some(before)) = (self.before_position, self.after_position) {
            if after < before {
                Err(CoreError::AfterSmallerBefore)?
            }
        }

        let order = if self.after_position.is_none() && self.before_position.is_some() {
            "DESC"
        } else {
            "ASC"
        };

        let query = format!(include_str!("../../sql/paginate_demons_by_position.sql"), order);

        // FIXME(sqlx) once CITEXT is supported
        let mut stream = sqlx::query(&query)
            .bind(self.before_position)
            .bind(self.after_position)
            .bind(self.name.as_ref().map(|s| s.as_str()))
            .bind(self.requirement)
            .bind(self.requirement_lt)
            .bind(self.requirement_gt)
            .bind(self.verifier_id)
            .bind(self.verifier_name.as_ref().map(|s| s.as_str()))
            .bind(self.publisher_id)
            .bind(self.publisher_name.as_ref().map(|s| s.as_str()))
            .bind(self.name_contains.as_ref().map(|s| s.as_str()))
            .bind(self.limit.unwrap_or(50) as i32 + 1)
            .fetch(connection);

        let mut demons = Vec::new();

        while let Some(row) = stream.next().await {
            let row = row?;

            let video: Option<String> = row.get("video");

            demons.push(Demon {
                base: MinimalDemon {
                    id: row.get("demon_id"),
                    name: row.get("demon_name"),
                    position: row.get("position"),
                },
                requirement: row.get("requirement"),
                video,
                publisher: DatabasePlayer {
                    id: row.get("publisher_id"),
                    name: row.get("publisher_name"),
                    banned: row.get("publisher_banned"),
                },
                verifier: DatabasePlayer {
                    id: row.get("verifier_id"),
                    name: row.get("verifier_name"),
                    banned: row.get("verifier_banned"),
                },
                level_id: row.get::<Option<i64>, _>("level_id").map(|id| id as u64),
            })
        }

        Ok(demons)
    }
}
