use crate::{
    demon::MinimalDemon,
    error::Result,
    player::DatabasePlayer,
    record::{MinimalRecordPD, RecordStatus},
};
use futures::StreamExt;
use pointercrate_core::{
    pagination::{Pagination, PaginationParameters},
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

impl Pagination for RecordPagination {
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

impl RecordPagination {
    /// Retrieves the page of records matching the pagination data in here
    ///
    /// Note that this method returns _one more record than requested_. This is used as a quick and
    /// dirty way to determine if further pages exist: If the additional record was returned, more
    /// pages obviously exist. This additional object is the last in the returned vector.
    ///
    /// Additionally, if _before_ is set, but not _after_, the page is returned in reverse order
    /// (the additional object stays the last)
    pub async fn page(&self, connection: &mut PgConnection) -> Result<Vec<MinimalRecordPD>> {
        self.params.validate()?;

        let order = self.params.order();

        let query = format!(include_str!("../../sql/paginate_records.sql"), order);

        let mut stream = sqlx::query(&query)
            .bind(self.params.before)
            .bind(self.params.after)
            .bind(self.progress)
            .bind(self.progress_lt)
            .bind(self.progress_gt)
            .bind(self.demon_position)
            .bind(self.demon_position_lt)
            .bind(self.demon_position_gt)
            .bind(self.status.map(|s| s.to_sql()))
            .bind(self.demon.as_deref())
            .bind(self.demon_id)
            .bind(&self.video)
            .bind(self.video == Some(None))
            .bind(self.player)
            .bind(self.submitter)
            .bind(self.params.limit + 1)
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

        Ok(records)
    }
}
