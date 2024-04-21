use crate::{
    demon::MinimalDemon,
    player::DatabasePlayer,
    record::{MinimalRecordPD, RecordStatus},
};
use futures::StreamExt;
use pointercrate_core::{
    first_and_last,
    pagination::{PageContext, Paginatable, PaginationParameters, PaginationQuery, __pagination_compat},
    util::{non_nullable, nullable},
};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgRow, PgConnection, Row};

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct RecordPagination {
    #[serde(flatten)]
    pub params: PaginationParameters,

    progress: Option<i16>,

    #[serde(default, deserialize_with = "non_nullable")]
    #[serde(rename = "progress__lt")]
    progress_lt: Option<i16>,

    #[serde(default, deserialize_with = "non_nullable")]
    #[serde(rename = "progress__gt")]
    progress_gt: Option<i16>,

    demon_position: Option<i16>,

    #[serde(default, deserialize_with = "non_nullable")]
    #[serde(rename = "demon_position__lt")]
    demon_position_lt: Option<i16>,

    #[serde(default, deserialize_with = "non_nullable")]
    #[serde(rename = "demon_position__gt")]
    demon_position_gt: Option<i16>,

    #[serde(default, deserialize_with = "non_nullable")]
    pub status: Option<RecordStatus>,

    #[serde(default, deserialize_with = "non_nullable")]
    pub player: Option<i32>,

    #[serde(default, deserialize_with = "non_nullable")]
    demon: Option<String>,

    #[serde(default, deserialize_with = "non_nullable")]
    demon_id: Option<i32>,

    #[serde(default, deserialize_with = "nullable")]
    video: Option<Option<String>>,

    #[serde(default, deserialize_with = "non_nullable")]
    pub submitter: Option<i32>,
}

impl PaginationQuery for RecordPagination {
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

impl Paginatable<RecordPagination> for MinimalRecordPD {
    first_and_last!("records");

    async fn page(query: &RecordPagination, connection: &mut PgConnection) -> Result<(Vec<MinimalRecordPD>, PageContext), sqlx::Error> {
        let order = query.params.order();

        let sql_query = format!(include_str!("../../sql/paginate_records.sql"), order);

        let mut stream = sqlx::query(&sql_query)
            .bind(query.params.before)
            .bind(query.params.after)
            .bind(query.progress)
            .bind(query.progress_lt)
            .bind(query.progress_gt)
            .bind(query.demon_position)
            .bind(query.demon_position_lt)
            .bind(query.demon_position_gt)
            .bind(query.status.map(|s| s.to_sql()))
            .bind(query.demon.as_deref())
            .bind(query.demon_id)
            .bind(&query.video)
            .bind(query.video == Some(None))
            .bind(query.player)
            .bind(query.submitter)
            .bind(query.params.limit + 1)
            .fetch(&mut *connection);

        let mut records = Vec::new();

        while let Some(row) = stream.next().await {
            let row: PgRow = row?;

            records.push(MinimalRecordPD {
                id: row.try_get("id")?,
                progress: row.try_get("progress")?,
                video: row.try_get("video")?,
                status: RecordStatus::from_sql(&row.try_get::<String, _>("status")?),
                player: DatabasePlayer {
                    id: row.try_get("player_id")?,
                    name: row.try_get("player_name")?,
                    banned: row.try_get("player_banned")?,
                },
                demon: MinimalDemon {
                    id: row.try_get("demon_id")?,
                    position: row.try_get("position")?,
                    name: row.try_get("demon_name")?,
                },
            })
        }

        Ok(__pagination_compat(&query.params, records))
    }

    fn pagination_id(&self) -> i32 {
        self.id
    }
}
