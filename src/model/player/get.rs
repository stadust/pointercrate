use super::Player;
use crate::{error::PointercrateError, operation::Get, Result};
use diesel::{result::Error, PgConnection, RunQueryDsl};

impl<'a> Get<&'a str> for Player {
    fn get(name: &'a str, connection: &PgConnection) -> Result<Self> {
        match Player::by_name(name).first(connection) {
            Ok(player) => Ok(player),
            Err(Error::NotFound) =>
                Player::insert(connection, &name).map_err(PointercrateError::database),
            Err(err) => Err(PointercrateError::database(err)),
        }
    }
}

impl Get<i32> for Player {
    fn get(id: i32, connection: &PgConnection) -> Result<Self> {
        match Player::by_id(id).first(connection) {
            Ok(player) => Ok(player),
            Err(Error::NotFound) =>
                Err(PointercrateError::ModelNotFound {
                    model: "Player",
                    identified_by: id.to_string(),
                }),
            Err(err) => Err(PointercrateError::database(err)),
        }
    }
}