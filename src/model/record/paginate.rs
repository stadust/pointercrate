use super::{PartialRecord, RecordStatus};
use crate::{
    error::PointercrateError,
    model::Model,
    operation::{Paginate, Paginator},
    schema::records,
    Result,
};
use diesel::{pg::Pg, query_builder::BoxedSelectStatement, PgConnection, QueryDsl, RunQueryDsl};
use serde_derive::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RecordPagination {
    #[serde(rename = "before")]
    before_id: Option<i32>,

    #[serde(rename = "after")]
    after_id: Option<i32>,

    limit: Option<i64>,

    progress: Option<i16>,
    #[serde(rename = "progress__lt")]
    progress_lt: Option<i16>,
    #[serde(rename = "progress__gt")]
    progress_gt: Option<i16>,

    status: Option<RecordStatus>,

    player: Option<i32>,
    submitter: Option<i32>,
    demon: Option<String>,
    video: Option<String>,
}

impl Paginator for RecordPagination {
    type Model = PartialRecord;
    type PaginationColumn = records::id;
    type PaginationColumnType = i32;

    filter_method!(records[
        progress = progress,
        progress < progress_lt,
        progress > progress_gt,
        status_ = status,
        player = player,
        submitter = submitter,
        demon = demon,
        video = video
    ]);

    fn page(
        &self, last_on_page: Option<Self::PaginationColumnType>,
        first_on_page: Option<Self::PaginationColumnType>,
    ) -> Self {
        RecordPagination {
            before_id: last_on_page.map(|i| i + 1),
            after_id: first_on_page.map(|i| i - 1),
            ..self.clone()
        }
    }

    fn limit(&self) -> i64 {
        self.limit.unwrap_or(50)
    }

    fn before(&self) -> Option<i32> {
        self.before_id
    }

    fn after(&self) -> Option<i32> {
        self.after_id
    }
}

impl Paginate<RecordPagination> for PartialRecord {
    fn load(pagination: &RecordPagination, connection: &PgConnection) -> Result<Vec<Self>> {
        let mut query = pagination.filter(PartialRecord::boxed_all());

        filter!(query[
            records::id > pagination.after_id,
            records::id < pagination.before_id
        ]);

        query
            .limit(pagination.limit.unwrap_or(50))
            .load(connection)
            .map_err(PointercrateError::database)
    }
}
