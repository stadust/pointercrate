use super::Creators;
use crate::{error::PointercrateError, model::Player, operation::Get, Result};
use diesel::{result::Error, PgConnection, RunQueryDsl};

impl Get<String> for Creators {
    fn get(name: String, connection: &PgConnection) -> Result<Self> {
        super::creators_of(&name)
            .load(connection)
            .map(Creators)
            .map_err(PointercrateError::database)
    }
}
