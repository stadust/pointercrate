use crate::{
    demon::{Demon, MinimalDemon},
    player::DatabasePlayer,
};
use futures::stream::StreamExt;
use pointercrate_core::{
    first_and_last,
    pagination::{PageContext, Pagination, PaginationParameters, __pagination_compat},
    util::non_nullable,
};
use serde::{Deserialize, Serialize};
use sqlx::{PgConnection, Row};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DemonIdPagination {
    #[serde(flatten)]
    pub params: PaginationParameters,

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

impl Pagination for DemonIdPagination {
    type Item = Demon;

    fn parameters(&self) -> PaginationParameters {
        self.params
    }

    fn with_parameters(&self, parameters: PaginationParameters) -> Self {
        Self {
            params: parameters,
            ..self.clone()
        }
    }

    first_and_last!("demons");

    async fn page(&self, connection: &mut PgConnection) -> Result<(Vec<Demon>, PageContext), sqlx::Error> {
        let order = self.params.order();

        let query = format!(include_str!("../../sql/paginate_demons_by_id.sql"), order);

        // FIXME(sqlx) once CITEXT is supported
        let mut stream = sqlx::query(&query)
            .bind(self.params.before)
            .bind(self.params.after)
            .bind(self.name.as_deref())
            .bind(self.requirement)
            .bind(self.requirement_lt)
            .bind(self.requirement_gt)
            .bind(self.verifier_id)
            .bind(self.verifier_name.as_deref())
            .bind(self.publisher_id)
            .bind(self.publisher_name.as_deref())
            .bind(self.name_contains.as_deref())
            .bind(self.params.limit + 1)
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
                thumbnail: row.get("thumbnail"),
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

        Ok(__pagination_compat(&self.params, demons))
    }

    fn id_of(demon: &Demon) -> i32 {
        demon.base.id
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DemonPositionPagination {
    #[serde(flatten)]
    pub params: PaginationParameters,

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

impl Pagination for DemonPositionPagination {
    type Item = Demon;

    fn parameters(&self) -> PaginationParameters {
        self.params
    }

    fn with_parameters(&self, parameters: PaginationParameters) -> Self {
        Self {
            params: parameters,
            ..self.clone()
        }
    }

    first_and_last!("demons", "position");

    async fn page(&self, connection: &mut PgConnection) -> Result<(Vec<Demon>, PageContext), sqlx::Error> {
        let order = self.params.order();

        let query = format!(include_str!("../../sql/paginate_demons_by_position.sql"), order);

        // FIXME(sqlx) once CITEXT is supported
        let mut stream = sqlx::query(&query)
            .bind(self.params.before)
            .bind(self.params.after)
            .bind(self.name.as_deref())
            .bind(self.requirement)
            .bind(self.requirement_lt)
            .bind(self.requirement_gt)
            .bind(self.verifier_id)
            .bind(self.verifier_name.as_deref())
            .bind(self.publisher_id)
            .bind(self.publisher_name.as_deref())
            .bind(self.name_contains.as_deref())
            .bind(self.params.limit + 1)
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
                thumbnail: row.get("thumbnail"),
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

        Ok(__pagination_compat(&self.params, demons))
    }

    fn id_of(item: &Self::Item) -> i32 {
        item.base.position as i32
    }
}
