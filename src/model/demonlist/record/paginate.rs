use super::{Record, RecordStatus};
use crate::{
    citext::CiString,
    context::RequestContext,
    model::{demonlist::record::records_pd, Model},
    operation::{Paginate, Paginator, PaginatorQuery, TablePaginator},
    Result,
};
use diesel::{ExpressionMethods, QueryDsl};
use serde_derive::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RecordPagination {
    #[serde(rename = "before")]
    before_id: Option<i32>,

    #[serde(rename = "after")]
    after_id: Option<i32>,

    limit: Option<u8>,

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

impl TablePaginator for RecordPagination {
    type ColumnType = i32;
    type PaginationColumn = records_pd::id;
    type Table = records_pd::table;

    fn query(&self, ctx: RequestContext) -> PaginatorQuery<records_pd::table> {
        let mut query = Record::boxed_all();

        filter!(query[
            records_pd::progress = self.progress,
            records_pd::progress < self.progress_lt,
            records_pd::progress > self.progress_gt,
            records_pd::status_ = self.status,
            records_pd::player_id = self.player,
            records_pd::demon_name = self.demon,
            records_pd::video = self.video
        ]);

        match ctx.is_list_mod() {
            true => filter!(query[records_pd::submitter_id = self.submitter]),
            false => query = query.filter(records_pd::status_.eq(RecordStatus::Approved)),
        };

        query
    }
}

delegate_to_table_paginator!(RecordPagination);

impl Paginate<RecordPagination> for Record {
    fn load(pagination: &RecordPagination, ctx: RequestContext) -> Result<Vec<Self>> {
        let mut query = pagination.query(ctx);

        filter!(query[
            records_pd::id > pagination.after_id,
            records_pd::id < pagination.before_id,
            records_pd::position = pagination.demon_position,
            records_pd::position < pagination.demon_position_lt,
            records_pd::position > pagination.demon_position_gt
        ]);

        let mut records: Vec<Record> =
            pagination_result!(query, pagination, records_pd::id, ctx.connection())?;

        if !ctx.is_list_mod() {
            for record in &mut records {
                record.submitter = None;
            }
        }

        Ok(records)
    }
}
