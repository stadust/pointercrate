use super::Submitter;
use crate::{
    context::RequestContext,
    model::Model,
    operation::{Paginate, Paginator, PaginatorQuery, TablePaginator},
    schema::submitters,
    Result,
};
use diesel::{ExpressionMethods, QueryDsl};
use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct SubmitterPagination {
    #[serde(rename = "before")]
    before_id: Option<i32>,

    #[serde(rename = "after")]
    after_id: Option<i32>,

    limit: Option<u8>,

    banned: Option<bool>,
}

impl TablePaginator for SubmitterPagination {
    type ColumnType = i32;
    type PaginationColumn = submitters::submitter_id;
    type Table = submitters::table;

    fn query(&self, _: RequestContext) -> PaginatorQuery<submitters::table> {
        let mut query = Submitter::boxed_all();

        if let Some(banned) = self.banned {
            query = query.filter(submitters::banned.eq(banned));
        }

        // FIXME: figure it out
        //query
        unimplemented!()
    }
}

delegate_to_table_paginator!(SubmitterPagination);

impl Paginate<SubmitterPagination> for Submitter {
    fn load(pagination: &SubmitterPagination, ctx: RequestContext) -> Result<Vec<Self>> {
        ctx.check_permissions(perms!(ListAdministrator))?;

        let mut query = pagination.query(ctx);

        /*filter!(query[
            submitters::submitter_id > pagination.after_id,
            submitters::submitter_id < pagination.before_id
        ]);

        pagination_result!(
            query,
            pagination,
            submitters::submitter_id,
            ctx.connection()
        )*/
        unimplemented!()
    }
}
