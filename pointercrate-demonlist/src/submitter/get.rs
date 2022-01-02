use crate::{
    error::{DemonlistError, Result},
    submitter::Submitter,
};
use sqlx::{Error, PgConnection};
use std::net::IpAddr;

impl Submitter {
    pub async fn by_id(id: i32, connection: &mut PgConnection) -> Result<Submitter> {
        let result = sqlx::query!("SELECT submitter_id, banned FROM submitters WHERE submitter_id = $1", id)
            .fetch_one(connection)
            .await;

        match result {
            Ok(row) => Ok(Submitter { id, banned: row.banned }),
            Err(Error::RowNotFound) => Err(DemonlistError::SubmitterNotFound { id }),
            Err(err) => Err(err.into()),
        }
    }

    pub async fn by_ip(ip: IpAddr, connection: &mut PgConnection) -> Result<Option<Submitter>> {
        Ok(sqlx::query!(
            "SELECT submitter_id, banned FROM submitters WHERE ip_address = cast($1::text as inet)",
            ip.to_string()
        )
        .fetch_optional(&mut *connection)
        .await?
        .map(|row| {
            Submitter {
                id: row.submitter_id,
                banned: row.banned,
            }
        }))
    }
}
