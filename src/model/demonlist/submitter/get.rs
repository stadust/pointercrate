use super::Submitter;
use crate::{
    error::PointercrateError,
    ratelimit::{PreparedRatelimits, RatelimitScope},
    Result,
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
            Err(Error::NotFound) =>
                Err(PointercrateError::ModelNotFound {
                    model: "Submitter",
                    identified_by: id.to_string(),
                }),
            Err(err) => Err(err.into()),
        }
    }

    pub async fn by_ip_or_create(
        ip: IpAddr, connection: &mut PgConnection, ratelimits: Option<PreparedRatelimits<'_>>,
    ) -> Result<Submitter> {
        let optional_row = sqlx::query!(
            "SELECT submitter_id, banned FROM submitters WHERE ip_address = cast($1::text as inet)",
            ip.to_string()
        )
        .fetch_optional(connection)
        .await?;

        match optional_row {
            Some(row) =>
                Ok(Submitter {
                    id: row.submitter_id,
                    banned: row.banned,
                }),
            None => {
                if let Some(ratelimits) = ratelimits {
                    ratelimits.check(RatelimitScope::NewSubmitter)?;
                }

                let id = sqlx::query!(
                    "INSERT INTO submitters (ip_address) VALUES (cast($1::text as inet)) RETURNING submitter_id",
                    ip.to_string()
                )
                .fetch_one(connection)
                .await?
                .submitter_id;

                Ok(Submitter { id, banned: false })
            },
        }
    }
}
