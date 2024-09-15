use crate::{
    demon::{Demon, MinimalDemon},
    player::DatabasePlayer,
};
use futures::stream::StreamExt;
use pointercrate_core::{
    first_and_last,
    pagination::{PageContext, Paginatable, PaginationParameters, PaginationQuery, __pagination_compat},
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
    level_id: Option<i64>,

    #[serde(default, deserialize_with = "non_nullable")]
    #[serde(rename = "requirement__gt")]
    requirement_gt: Option<i16>,

    #[serde(default, deserialize_with = "non_nullable")]
    #[serde(rename = "requirement__lt")]
    requirement_lt: Option<i16>,
}

impl PaginationQuery for DemonIdPagination {
    fn parameters(&self) -> PaginationParameters {
        self.params
    }

    fn with_parameters(&self, parameters: PaginationParameters) -> Self {
        Self {
            params: parameters,
            ..self.clone()
        }
    }
}

impl Paginatable<DemonIdPagination> for Demon {
    first_and_last!("demons");

    async fn page(query: &DemonIdPagination, connection: &mut PgConnection) -> Result<(Vec<Demon>, PageContext), sqlx::Error> {
        let order = query.params.order();

        let sql_query = format!(include_str!("../../sql/paginate_demons_by_id.sql"), order);

        // FIXME(sqlx) once CITEXT is supported
        let mut stream = sqlx::query(&sql_query)
            .bind(query.params.before)
            .bind(query.params.after)
            .bind(query.name.as_deref())
            .bind(query.requirement)
            .bind(query.requirement_lt)
            .bind(query.requirement_gt)
            .bind(query.verifier_id)
            .bind(query.verifier_name.as_deref())
            .bind(query.publisher_id)
            .bind(query.publisher_name.as_deref())
            .bind(query.name_contains.as_deref())
            .bind(query.level_id)
            .bind(query.params.limit + 1)
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

        Ok(__pagination_compat(&query.params, demons))
    }

    fn pagination_id(&self) -> i32 {
        self.base.id
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct DemonPositionPagination {
    #[serde(flatten)]
    pub params: PaginationParameters,

    #[serde(default, deserialize_with = "non_nullable")]
    pub name: Option<String>,

    #[serde(default, deserialize_with = "non_nullable")]
    pub name_contains: Option<String>,

    #[serde(default, deserialize_with = "non_nullable")]
    pub requirement: Option<i16>,

    #[serde(default, deserialize_with = "non_nullable")]
    pub verifier_id: Option<i32>,
    #[serde(default, deserialize_with = "non_nullable")]
    pub publisher_id: Option<i32>,

    #[serde(default, deserialize_with = "non_nullable")]
    pub verifier_name: Option<String>,
    #[serde(default, deserialize_with = "non_nullable")]
    pub publisher_name: Option<String>,

    #[serde(default, deserialize_with = "non_nullable")]
    pub level_id: Option<i64>,

    #[serde(default, deserialize_with = "non_nullable")]
    #[serde(rename = "requirement__gt")]
    pub requirement_gt: Option<i16>,

    #[serde(default, deserialize_with = "non_nullable")]
    #[serde(rename = "requirement__lt")]
    pub requirement_lt: Option<i16>,
}

impl PaginationQuery for DemonPositionPagination {
    fn parameters(&self) -> PaginationParameters {
        self.params
    }

    fn with_parameters(&self, parameters: PaginationParameters) -> Self {
        Self {
            params: parameters,
            ..self.clone()
        }
    }
}

impl Paginatable<DemonPositionPagination> for Demon {
    first_and_last!("demons", "position");

    async fn page(query: &DemonPositionPagination, connection: &mut PgConnection) -> Result<(Vec<Demon>, PageContext), sqlx::Error> {
        let order = query.params.order();

        let sql_query = format!(include_str!("../../sql/paginate_demons_by_position.sql"), order);

        // FIXME(sqlx) once CITEXT is supported
        let mut stream = sqlx::query(&sql_query)
            .bind(query.params.before)
            .bind(query.params.after)
            .bind(query.name.as_deref())
            .bind(query.requirement)
            .bind(query.requirement_lt)
            .bind(query.requirement_gt)
            .bind(query.verifier_id)
            .bind(query.verifier_name.as_deref())
            .bind(query.publisher_id)
            .bind(query.publisher_name.as_deref())
            .bind(query.name_contains.as_deref())
            .bind(query.level_id)
            .bind(query.params.limit + 1)
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

        Ok(__pagination_compat(&query.params, demons))
    }

    fn pagination_id(&self) -> i32 {
        self.base.position as i32
    }
}
