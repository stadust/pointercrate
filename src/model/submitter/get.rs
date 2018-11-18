use super::Submitter;
use crate::{error::PointercrateError, operation::Get, Result};
use diesel::{result::Error, PgConnection, RunQueryDsl};
use ipnetwork::IpNetwork;

impl Get<IpNetwork> for Submitter {
    fn get(ip: IpNetwork, connection: &PgConnection) -> Result<Self> {
        match Submitter::by_ip(&ip).first(connection) {
            Ok(submitter) => Ok(submitter),
            Err(Error::NotFound) =>
                Submitter::insert(connection, &ip).map_err(PointercrateError::database),
            Err(err) => Err(PointercrateError::database(err)),
        }
    }
}
