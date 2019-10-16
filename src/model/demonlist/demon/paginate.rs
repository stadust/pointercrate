use crate::{
    citext::CiString,
    context::RequestContext,
    model::{
        demonlist::{demon::demons_pv, Demon},
        Model,
    },
    operation::{Paginate, Paginator, PaginatorQuery, TablePaginator},
    Result,
};
use diesel::QueryDsl;
use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DemonPagination {
    #[serde(rename = "before")]
    before_position: Option<i16>,

    #[serde(rename = "after")]
    after_position: Option<i16>,

    limit: Option<u8>,

    name: Option<CiString>,

    requirement: Option<i16>,

    verifier_id: Option<i32>,
    publisher_id: Option<i32>,

    verifier_name: Option<CiString>,
    publisher_name: Option<CiString>,

    #[serde(rename = "requirement__gt")]
    requirement_gt: Option<i16>,

    #[serde(rename = "requirement__lt")]
    requirement_lt: Option<i16>,
}

impl TablePaginator for DemonPagination {
    type ColumnType = i16;
    type PaginationColumn = demons_pv::position;
    type Table = demons_pv::table;

    fn query(&self, _: RequestContext) -> PaginatorQuery<demons_pv::table> {
        let mut query = Demon::boxed_all();

        filter!(query[
            demons_pv::name = self.name,
            demons_pv::requirement = self.requirement,
            demons_pv::requirement < self.requirement_lt,
            demons_pv::requirement > self.requirement_gt,
            demons_pv::verifier_id = self.verifier_id,
            demons_pv::publisher_id = self.publisher_id,
            demons_pv::verifier_name = self.verifier_name,
            demons_pv::publisher_name = self.publisher_name
        ]);

        query
    }
}

delegate_to_table_paginator!(DemonPagination, limit, before_position, after_position);

impl Paginate<DemonPagination> for Demon {
    fn load(pagination: &DemonPagination, ctx: RequestContext) -> Result<Vec<Self>> {
        let mut query = pagination.query(ctx);

        filter!(query[
            demons_pv::position > pagination.after_position,
            demons_pv::position < pagination.before_position
        ]);

        pagination_result!(
            query,
            pagination,
            before_position,
            after_position,
            demons_pv::position,
            ctx.connection()
        )
    }
}
