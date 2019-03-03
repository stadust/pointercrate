use super::{Submitter, SubmitterWithRecords};
use crate::{
    error::PointercrateError,
    model::user::User,
    operation::Get,
    permissions::{self, AccessRestrictions},
    Result,
};
use diesel::{result::Error, PgConnection, RunQueryDsl};
use ipnetwork::IpNetwork;

impl Get<IpNetwork> for Submitter {
    fn get(ip: IpNetwork, connection: &PgConnection) -> Result<Self> {
        match Submitter::by_ip(&ip).first(connection) {
            Ok(submitter) => Ok(submitter),
            Err(Error::NotFound) =>
                Submitter::insert(&ip, connection).map_err(PointercrateError::database),
            Err(err) => Err(PointercrateError::database(err)),
        }
    }
}

impl Get<i32> for Submitter {
    fn get(id: i32, connection: &PgConnection) -> Result<Self> {
        match Submitter::by_id(id).first(connection) {
            Ok(submitter) => Ok(submitter),
            Err(Error::NotFound) =>
                Err(PointercrateError::ModelNotFound {
                    model: "Submitter",
                    identified_by: id.to_string(),
                }),
            Err(err) => Err(PointercrateError::database(err)),
        }
    }
}

impl AccessRestrictions for Submitter {
    fn pre_access(user: Option<&User>) -> Result<()> {
        permissions::demand(perms!(ListModerator or ListAdministrator), user)
    }
}

impl<T> Get<T> for SubmitterWithRecords
where
    Submitter: Get<T>,
{
    fn get(t: T, connection: &PgConnection) -> Result<Self> {
        let submitter = Submitter::get(t, connection)?;

        Ok(SubmitterWithRecords {
            records: Get::get(&submitter, connection)?,
            submitter,
        })
    }
}

impl AccessRestrictions for SubmitterWithRecords {
    fn pre_access(user: Option<&User>) -> Result<()> {
        permissions::demand(perms!(ListModerator or ListAdministrator), user)
    }
}
