use super::Creator;
use crate::{
    error::PointercrateError,
    model::{Demon, Player},
    operation::{Get, Post},
    schema::creators,
    Result,
};
use diesel::{insert_into, Connection, PgConnection, RunQueryDsl};

#[derive(Debug, Insertable)]
#[table_name = "creators"]
struct NewCreator<'a> {
    demon: &'a str,
    creator: i32,
}

impl<'a> Post<(String, String)> for Creator {
    fn create_from(
        (demon, player): (String, String), connection: &PgConnection,
    ) -> Result<Creator> {
        connection.transaction(|| {
            let demon = Demon::get(demon, connection)?;
            let player = Player::get(player, connection)?;

            insert_into(creators::table)
                .values(&NewCreator {
                    demon: &demon.name,
                    creator: player.id,
                })
                .get_result(connection)
                .map_err(PointercrateError::database)
        })
    }
}
