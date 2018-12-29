use super::PartialDemon;
use crate::{
    error::PointercrateError,
    model::Model,
    operation::{Paginate, Paginator},
    schema::{demons, players},
    Result,
};
use diesel::{
    expression::Expression, pg::Pg, query_builder::BoxedSelectStatement, ExpressionMethods,
    JoinOnDsl, PgConnection, QueryDsl, RunQueryDsl,
};
use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DemonPagination {
    #[serde(rename = "before")]
    before_position: Option<i16>,

    #[serde(rename = "after")]
    after_position: Option<i16>,

    limit: Option<i64>,

    name: Option<String>,

    requirement: Option<i16>,

    #[serde(rename = "requirement__gt")]
    requirement_gt: Option<i16>,

    #[serde(rename = "requirement__lt")]
    requirement_lt: Option<i16>,
}

impl DemonPagination {
    filter_method!(demons[
        name = name,
        requirement = requirement,
        requirement < requirement_lt,
        requirement > requirement_gt
    ]);
}

impl Paginator for DemonPagination {
    type Model = PartialDemon;

    navigation!(demons, position, i16, before_position, after_position);
}

impl Paginate<DemonPagination> for PartialDemon {
    fn load(pagination: &DemonPagination, connection: &PgConnection) -> Result<Vec<Self>> {
        let mut query = pagination.filter(PartialDemon::boxed_all());

        filter!(query[
            demons::position > pagination.after_position,
            demons::position < pagination.before_position
        ]);

        query
            .limit(pagination.limit.unwrap_or(50))
            .load(connection)
            .map_err(PointercrateError::database)
    }
}
