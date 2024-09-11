use crate::{error::Result, submitter::Submitter};
use sqlx::PgConnection;
use std::net::IpAddr;

impl Submitter {
    pub async fn create_submitter(ip: IpAddr, connection: &mut PgConnection) -> Result<Submitter> {
        let id = sqlx::query!(
            "INSERT INTO submitters (ip_address) VALUES (cast($1::text as inet)) RETURNING submitter_id",
            ip.to_string()
        )
        .fetch_one(connection)
        .await?
        .submitter_id;

        Ok(Submitter { id, banned: false })
    }
}
