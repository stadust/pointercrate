use crate::{
    cistring::CiString,
    error::PointercrateError,
    model::demonlist::{
        demon::MinimalDemon,
        player::DatabasePlayer,
        record::{MinimalRecordPD, RecordStatus},
    },
    util::{non_nullable, nullable},
    Result,
};
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgRow, PgConnection, Row};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RecordPagination {
    #[serde(default, deserialize_with = "non_nullable")]
    #[serde(rename = "before")]
    pub before_id: Option<i32>,

    #[serde(default, deserialize_with = "non_nullable")]
    #[serde(rename = "after")]
    pub after_id: Option<i32>,

    #[serde(default, deserialize_with = "non_nullable")]
    limit: Option<u8>,

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

    player: Option<i32>,

    #[serde(default, deserialize_with = "non_nullable")]
    demon: Option<CiString>,

    #[serde(default, deserialize_with = "non_nullable")]
    demon_id: Option<i32>,

    #[serde(default, deserialize_with = "nullable")]
    video: Option<Option<String>>,
}

impl RecordPagination {
    pub async fn page(&self, connection: &mut PgConnection) -> Result<Vec<MinimalRecordPD>> {
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

        let mut stream = sqlx::query_as(include_str!("../../../../sql/paginate_records.sql"))
            .bind(self.before_id)
            .bind(self.after_id)
            .bind(self.progress)
            .bind(self.progress_lt)
            .bind(self.progress_gt)
            .bind(self.demon_position)
            .bind(self.demon_position_lt)
            .bind(self.demon_position_gt)
            .bind(self.status.map(|s| s.to_sql()))
            .bind(&self.demon)
            .bind(self.demon_id)
            .bind(&self.video)
            .bind(self.video == Some(None))
            .bind(self.limit.unwrap_or(50) as i32)
            .fetch(connection);

        let mut records = Vec::new();

        while let Some(row) = stream.next().await {
            let row: PgRow = row?;

            records.push(MinimalRecordPD {
                id: row.get("id"),
                progress: row.get("progress"),
                video: row.get("video"),
                status: RecordStatus::from_sql(&row.get::<String, _>("status")),
                player: DatabasePlayer {
                    id: row.get("player_id"),
                    name: CiString(row.get("player_name")),
                    banned: row.get("player_banned"),
                },
                demon: MinimalDemon {
                    id: row.get("demon_id"),
                    position: row.get("position"),
                    name: CiString(row.get("demon_name")),
                },
            })
        }

        Ok(records)
    }
}
