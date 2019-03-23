use super::{Record, RecordStatus};
use crate::{
    citext::CiString,
    context::RequestContext,
    error::PointercrateError,
    model::Model,
    operation::{Paginate, Paginator},
    schema::{demons, records},
    Result,
};
use diesel::{pg::Pg, query_builder::BoxedSelectStatement, QueryDsl, RunQueryDsl};
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

    demon_position: Option<i16>,
    #[serde(rename = "demon_position__lt")]
    demon_position_lt: Option<i16>,
    #[serde(rename = "demon_position__gt")]
    demon_position_gt: Option<i16>,

    status: Option<RecordStatus>,

    player: Option<i32>,
    submitter: Option<i32>,
    demon: Option<CiString>,
    video: Option<String>,
}

impl Paginator for RecordPagination {
    type Model = Record;
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

impl Paginate<RecordPagination> for Record {
    fn load(pagination: &RecordPagination, ctx: RequestContext) -> Result<Vec<Self>> {
        if pagination.limit() > 100 || pagination.limit() < 1 {
            return Err(PointercrateError::InvalidPaginationLimit)
        }

        let mut query = pagination.filter(Record::boxed_all());

        filter!(query[
            records::id > pagination.after_id,
            records::id < pagination.before_id,
            demons::position = pagination.demon_position,
            demons::position < pagination.demon_position_lt,
            demons::position > pagination.demon_position_gt
        ]);

        if !ctx.is_list_mod() {
            query = query.filter(records::status_.eq(RecordStatus::Approved));
        }

        let mut records: Vec<Record> =
            pagination_result!(query, pagination, records::id, ctx.connection())?;

        if !ctx.is_list_mod() {
            for record in &mut records {
                record.submitter = None;
            }
        }

        Ok(records)
    }
}
