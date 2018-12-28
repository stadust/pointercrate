use super::PartialDemon;
use crate::{
    error::PointercrateError,
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
    type QuerySource = diesel::query_source::joins::JoinOn<
        diesel::query_source::joins::Join<
            demons::table,
            players::table,
            diesel::query_source::joins::Inner,
        >,
        diesel::expression::operators::Eq<demons::columns::publisher, players::columns::id>,
    >;
    type Selection = (demons::name, demons::position, players::name);

    navigation!(demons, position, i16, before_position, after_position);

    fn base<'a>(
    ) -> BoxedSelectStatement<'a, <Self::Selection as Expression>::SqlType, Self::QuerySource, Pg>
    {
        demons::table
            .inner_join(players::table.on(demons::publisher.eq(players::id)))
            .select((demons::name, demons::position, players::name))
            .into_boxed()
    }
}

impl Paginate<DemonPagination> for PartialDemon {
    fn load(pagination: &DemonPagination, connection: &PgConnection) -> Result<Vec<Self>> {
        let mut query = pagination.filter(DemonPagination::base());

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
