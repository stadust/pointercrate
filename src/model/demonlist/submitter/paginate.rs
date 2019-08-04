use super::Submitter;
use crate::{
    context::RequestContext,
    error::PointercrateError,
    model::Model,
    operation::{Paginate, Paginator},
    schema::submitters,
    Result,
};
use diesel::{pg::Pg, query_builder::BoxedSelectStatement, QueryDsl, RunQueryDsl};
use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct SubmitterPagination {
    #[serde(rename = "before")]
    before_id: Option<i32>,

    #[serde(rename = "after")]
    after_id: Option<i32>,

    limit: Option<i64>,

    banned: Option<bool>,
}

impl Paginator for SubmitterPagination {
    type Model = Submitter;
    type PaginationColumn = submitters::submitter_id;
    type PaginationColumnType = i32;

    filter_method!(submitters[banned = banned]);

    fn page(
        &self,
        last_on_page: Option<Self::PaginationColumnType>,
        first_on_page: Option<Self::PaginationColumnType>,
    ) -> Self {
        SubmitterPagination {
            before_id: last_on_page.map(|i| i + 1),
            after_id: first_on_page.map(|i| i - 1),
            banned: self.banned,
            limit: self.limit,
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

impl Paginate<SubmitterPagination> for Submitter {
    fn load(pagination: &SubmitterPagination, ctx: RequestContext) -> Result<Vec<Self>> {
        ctx.check_permissions(perms!(ListAdministrator))?;

        let mut query = pagination.filter(Submitter::boxed_all(), ctx);

        filter!(query[
            submitters::submitter_id > pagination.after_id,
            submitters::submitter_id < pagination.before_id
        ]);

        pagination_result!(
            query,
            pagination,
            submitters::submitter_id,
            ctx.connection()
        )
    }
}
