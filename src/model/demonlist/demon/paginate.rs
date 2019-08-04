use super::PartialDemon;
use crate::{
    citext::CiString,
    context::RequestContext,
    error::PointercrateError,
    model::Model,
    operation::{Paginate, Paginator},
    schema::demons,
    Result,
};
use diesel::{pg::Pg, query_builder::BoxedSelectStatement, QueryDsl, RunQueryDsl};
use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DemonPagination {
    #[serde(rename = "before")]
    before_position: Option<i16>,

    #[serde(rename = "after")]
    after_position: Option<i16>,

    limit: Option<i64>,

    name: Option<CiString>,

    requirement: Option<i16>,

    #[serde(rename = "requirement__gt")]
    requirement_gt: Option<i16>,

    #[serde(rename = "requirement__lt")]
    requirement_lt: Option<i16>,
}

impl Paginator for DemonPagination {
    type Model = PartialDemon;
    type PaginationColumn = demons::position;
    type PaginationColumnType = i16;

    filter_method!(demons[
        name = name,
        requirement = requirement,
        requirement < requirement_lt,
        requirement > requirement_gt
    ]);

    fn page(
        &self,
        last_on_page: Option<Self::PaginationColumnType>,
        first_on_page: Option<Self::PaginationColumnType>,
    ) -> Self {
        DemonPagination {
            before_position: last_on_page.map(|i| i + 1),
            after_position: first_on_page.map(|i| i - 1),
            ..self.clone()
        }
    }

    fn limit(&self) -> i64 {
        self.limit.unwrap_or(50)
    }

    fn before(&self) -> Option<i16> {
        self.before_position
    }

    fn after(&self) -> Option<i16> {
        self.after_position
    }
}

impl Paginate<DemonPagination> for PartialDemon {
    fn load(pagination: &DemonPagination, ctx: RequestContext) -> Result<Vec<Self>> {
        let mut query = pagination.filter(PartialDemon::boxed_all(), ctx);

        filter!(query[
            demons::position > pagination.after_position,
            demons::position < pagination.before_position
        ]);

        pagination_result!(
            query,
            pagination,
            before_position,
            after_position,
            demons::position,
            ctx.connection()
        )
    }
}
