use super::{FullSubmitter, Submitter};
use crate::{
    model::demonlist::record::submitted_by,
    ratelimit::{PreparedRatelimits, RatelimitScope},
    Result,
};
use sqlx::PgConnection;
use std::net::IpAddr;

impl Submitter {
    pub async fn by_id(id: i32, connection: &mut PgConnection) -> Result<Submitter> {
        let row = sqlx::query!("SELECT submitter_id, banned FROM submitters WHERE submitter_id = $1", id)
            .fetch_one(connection)
            .await?;

        Ok(Submitter { id, banned: row.banned })
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

    pub async fn upgrade(self, connection: &mut PgConnection) -> Result<FullSubmitter> {
        Ok(FullSubmitter {
            records: submitted_by(&self, connection).await?,
            submitter: self,
        })
    }
}

impl FullSubmitter {
    pub async fn by_id(id: i32, connection: &mut PgConnection) -> Result<FullSubmitter> {
        Submitter::by_id(id, connection).await?.upgrade(connection).await
    }
}
