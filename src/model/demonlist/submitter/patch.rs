use crate::{model::demonlist::submitter::Submitter, util::non_nullable, Result};
use log::info;
use serde::Deserialize;
use sqlx::PgConnection;

#[derive(Debug, Deserialize)]
pub struct PatchSubmitter {
    #[serde(default, deserialize_with = "non_nullable")]
    banned: Option<bool>,
}

impl Submitter {
    pub async fn ban(&mut self, connection: &mut PgConnection) -> Result<()> {
        sqlx::query!("UPDATE submitters SET banned = true WHERE submitter_id = $1", self.id)
            .execute(connection)
            .await?;

        let deleted = sqlx::query!("DELETE FROM records WHERE submitter = $1 AND status_ = 'SUBMITTED'", self.id)
            .execute(connection)
            .await?;

        info!("Banning submitter {} caused deletion of {} submissions", self, deleted);

        self.banned = true;

        Ok(())
    }

    pub async fn unban(&mut self, connection: &mut PgConnection) -> Result<()> {
        sqlx::query!("UPDATE submitters SET banned = false WHERE submitter_id = $1", self.id)
            .execute(connection)
            .await?;

        self.banned = false;

        Ok(())
    }

    pub async fn apply_patch(mut self, patch: PatchSubmitter, connection: &mut PgConnection) -> Result<Self> {
        info!("Patching submitter {} with {:?}", self, patch);

        match patch.banned {
            Some(true) => self.ban(connection).await?,
            Some(false) => self.unban(connection).await?,
            _ => (),
        }

        Ok(self)
    }
}
