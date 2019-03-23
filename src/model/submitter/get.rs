use super::{Submitter, SubmitterWithRecords};
use crate::{context::RequestContext, error::PointercrateError, model::By, operation::Get, Result};
use diesel::{result::Error, PgConnection, RunQueryDsl};
use ipnetwork::IpNetwork;
use std::net::Ipv4Addr;

impl Get<()> for Submitter {
    fn get(_: (), ctx: RequestContext, connection: &PgConnection) -> Result<Self> {
        match ctx {
            RequestContext::Internal(_) =>
                Ok(Submitter {
                    id: 0,
                    ip: IpNetwork::V4(Ipv4Addr::new(127, 0, 0, 1).into()),
                    banned: false,
                }),
            RequestContext::External { ip, .. } =>
                match Submitter::by(&ip).first(connection) {
                    Ok(submitter) => Ok(submitter),
                    Err(Error::NotFound) =>
                        Submitter::insert(&ip, connection).map_err(PointercrateError::database),
                    Err(err) => Err(PointercrateError::database(err)),
                },
        }
    }
}

impl Get<i32> for Submitter {
    fn get(id: i32, ctx: RequestContext, connection: &PgConnection) -> Result<Self> {
        ctx.check_permissions(perms!(ListModerator or ListAdministrator))?;

        match Submitter::by(id).first(connection) {
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

impl<T> Get<T> for SubmitterWithRecords
where
    Submitter: Get<T>,
{
    fn get(t: T, ctx: RequestContext, connection: &PgConnection) -> Result<Self> {
        let submitter = Submitter::get(t, ctx, connection)?;

        Ok(SubmitterWithRecords {
            records: Get::get(&submitter, ctx, connection)?,
            submitter,
        })
    }
}
