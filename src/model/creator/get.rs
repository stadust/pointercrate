use super::{Creator, Creators};
use crate::{
    error::PointercrateError,
    model::{Demon, Player},
    operation::Get,
    schema::creators,
    Result,
};
use diesel::{
    expression::bound::Bound, pg::Pg, sql_types, ExpressionMethods, PgConnection, QueryDsl,
    QueryResult, Queryable, RunQueryDsl,
};

impl<'a> Get<&'a str> for Creators {
    fn get(name: &'a str, connection: &PgConnection) -> Result<Self> {
        super::creators_of(name)
            .load(connection)
            .map(Creators)
            .map_err(PointercrateError::database)
    }
}

impl Get<(i16, i32)> for Creator {
    fn get((demon_position, player_id): (i16, i32), connection: &PgConnection) -> Result<Self> {
        let demon = Demon::get(demon_position, connection)?;

        creators::table
            .select((creators::demon, creators::creator))
            .filter(creators::demon.eq(&demon.name))
            .filter(creators::creator.eq(&player_id))
            .get_result(connection)
            .map_err(PointercrateError::database)
    }
}
