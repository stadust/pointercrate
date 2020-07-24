use crate::{
    cistring::CiString,
    error::PointercrateError,
    model::{
        demonlist::{
            player::{DatabasePlayer, FullPlayer, Player},
            record::{approved_records_by, FullRecord},
        },
        nationality::Nationality,
    },
    util::{non_nullable, nullable},
    Result,
};
use log::info;
use serde::Deserialize;
use sqlx::PgConnection;

#[derive(Debug, Deserialize, Default)]
pub struct PatchPlayer {
    #[serde(default, deserialize_with = "non_nullable")]
    name: Option<CiString>,

    #[serde(default, deserialize_with = "non_nullable")]
    banned: Option<bool>,

    #[serde(default, deserialize_with = "nullable")]
    nationality: Option<Option<CiString>>,
}

impl FullPlayer {
    pub async fn apply_patch(mut self, patch: PatchPlayer, connection: &mut PgConnection) -> Result<Self> {
        if let Some(nationality) = patch.nationality {
            match nationality {
                Some(ident) =>
                    self.player
                        .set_nationality(Nationality::by_country_code_or_name(ident.as_ref(), connection).await?, connection)
                        .await?,
                None => self.player.reset_nationality(connection).await?,
            }
        }

        if let Some(banned) = patch.banned {
            if banned && !self.player.base.banned {
                self.player.base.ban(connection).await?;

                // self.records only contains approved records!
                self.records.clear();
            } else if !banned && self.player.base.banned {
                self.player.base.unban(connection).await?;
            }
        }

        if let Some(name) = patch.name {
            self.set_name(name, connection).await?;
        }

        Ok(self)
    }

    pub async fn set_name(&mut self, name: CiString, connection: &mut PgConnection) -> Result<()> {
        let name = CiString(name.trim().to_string());

        // Nothing to be done
        if name.eq_sensitive(self.player.base.name.as_ref()) {
            return Ok(())
        } else if name != self.player.base.name {
            // If they are equal case sensitively, we're only doing a cosmetic rename, which won't
            // even require a merge

            // try to see if a player with new name already exists
            match DatabasePlayer::by_name(name.as_ref(), connection).await {
                Ok(existing) => self.merge(existing, connection).await?,
                Err(PointercrateError::ModelNotFound { .. }) => (),
                Err(err) => return Err(err),
            }
        }

        sqlx::query!(
            "UPDATE players SET name = $1::text WHERE id = $2",
            name.to_string(),
            self.player.base.id
        )
        .execute(connection)
        .await?;

        self.player.base.name = name;

        Ok(())
    }

    /// Merges the given player into `Self`, deleting `with`.
    ///
    /// Note that this method **does not** rename `Self`
    pub async fn merge(&mut self, with: DatabasePlayer, connection: &mut PgConnection) -> Result<()> {
        info!("Merging player {} with player {}", self, with);
        // First, delete duplicate creator entries

        let deleted = sqlx::query!(
            "DELETE FROM creators AS c1 WHERE c1.creator = $2 AND EXISTS (SELECT 1 FROM creators AS c2 WHERE c2.demon = c1.demon AND \
             c2.creator = $1)",
            self.player.base.id,
            with.id
        )
        .execute(connection)
        .await?;

        info!("Deleted {} duplicate creator entries while merging {} and {}", deleted, self, with);

        // Transfer all other creator entries over
        let updated = sqlx::query!("UPDATE creators SET creator = $1 WHERE creator = $2", self.player.base.id, with.id)
            .execute(connection)
            .await?;

        info!("Transferred {} creator entries from {} to {}", updated, with, self);

        // Transfer over verifier and publisher entries

        let updated_verifiers = sqlx::query!("UPDATE demons SET verifier = $1 WHERE verifier = $2", self.player.base.id, with.id)
            .execute(connection)
            .await?;
        let updated_publishers = sqlx::query!(
            "UPDATE demons SET publisher = $1 WHERE publisher = $2",
            self.player.base.id,
            with.id
        )
        .execute(connection)
        .await?;

        info!(
            "Transferred over {} verifier and {} publisher entires from {} to {}",
            updated_verifiers, updated_publishers, with, self
        );

        // Alright so merging records is HARD. We already implemented it over in the record patching, so
        // while somewhat inefficient maybe, we'll just call that code for each record of the current player
        for row in sqlx::query!("SELECT id FROM records WHERE player = $1", self.player.base.id)
            .fetch_all(connection)
            .await?
        {
            // FIXME: this is really inefficient and can be made a lot faster by simple moving around some code
            // in the FullRecord impls
            let mut record = FullRecord::by_id(row.id, connection).await?;
            info!("Moving record {} over to new player {}", record, self.player.base);
            record.set_player(self.player.base.clone(), connection).await?
        }

        self.records = approved_records_by(&self.player.base, connection).await?;

        // Transfer all records over, now that they're unique
        let updated = sqlx::query!("UPDATE records SET player = $1 WHERE player = $2", self.player.base.id, with.id)
            .execute(connection)
            .await?;

        info!("Moved {} records from {} to {}", updated, with, self);

        // Delete the second player
        sqlx::query!("DELETE FROM players WHERE id = $1", with.id)
            .execute(connection)
            .await?;

        Ok(())
    }
}

impl Player {
    pub async fn reset_nationality(&mut self, connection: &mut PgConnection) -> Result<()> {
        sqlx::query!("UPDATE players SET nationality = NULL WHERE id = $1", self.base.id)
            .execute(connection)
            .await?;

        self.nationality = None;

        Ok(())
    }

    pub async fn set_nationality(&mut self, nationality: Nationality, connection: &mut PgConnection) -> Result<()> {
        sqlx::query!(
            "UPDATE players SET nationality = $1::text WHERE id = $2",
            nationality.iso_country_code,
            self.base.id
        )
        .execute(connection)
        .await?;

        self.nationality = Some(nationality);

        Ok(())
    }
}

impl DatabasePlayer {
    pub async fn unban(&mut self, connection: &mut PgConnection) -> Result<()> {
        sqlx::query!("UPDATE players SET banned = false WHERE id=$1", self.id)
            .execute(connection)
            .await?;

        self.banned = false;

        Ok(())
    }

    pub async fn ban(&mut self, connection: &mut PgConnection) -> Result<()> {
        // Delete all submissions for this player
        let deleted = sqlx::query!(
            "DELETE FROM records WHERE player = $1 AND (status_ = 'SUBMITTED' OR status_ = 'UNDER_CONSIDERATION')",
            self.id
        )
        .execute(connection)
        .await?;

        info!("Deleted {} submissions while banning {}", deleted, self);

        // We can simply reject all accepted records here! All submitted records were deleted above, and we
        // don't have to worry about conflicts with existing rejected record when setting status to
        // 'rejected' since rejected records are globally unique!

        // Now, reject all previously accepted records
        let updated = sqlx::query!("UPDATE records SET status_ = 'REJECTED' WHERE player = $1", self.id)
            .execute(connection)
            .await?;

        info!("Rejected {} records while banning {}", updated, self);

        // Actually ban the player
        sqlx::query!("UPDATE players SET banned = true WHERE id = $1", self.id)
            .execute(connection)
            .await?;

        self.banned = true;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        cistring::{CiStr, CiString},
        model::demonlist::{
            player::{DatabasePlayer, PatchPlayer, Player},
            record::{RecordPagination, RecordStatus},
        },
    };

    #[actix_rt::test]
    async fn test_cosmetic_rename() {
        let mut connection = crate::test::test_setup().await;
        let player_id = DatabasePlayer::by_name(CiStr::from_str("stardust1971"), &mut connection)
            .await
            .unwrap()
            .id;
        let player = Player::by_id(player_id, &mut connection).await.unwrap();
        let player = player.upgrade(&mut connection).await.unwrap();
        let player_before = Player::by_id(player_id, &mut connection).await.unwrap();
        let player_before = player_before.upgrade(&mut connection).await.unwrap();

        let patch = PatchPlayer {
            name: Some(CiString("STARDUST1971".to_owned())),
            ..Default::default()
        };

        let player_after = player.apply_patch(patch, &mut connection).await;

        assert!(player_after.is_ok(), "{:?}", player_after.unwrap_err());
        assert_eq!(player_after.unwrap(), player_before);
    }

    #[actix_rt::test]
    async fn test_ban_player() {
        let mut connection = crate::test::test_setup().await;
        let player_id = DatabasePlayer::by_name(CiStr::from_str("stardust1971"), &mut connection)
            .await
            .unwrap()
            .id;
        let player = Player::by_id(player_id, &mut connection).await.unwrap();
        let mut player = player.upgrade(&mut connection).await.unwrap();

        // using pagination here is okay, cause the test database doesn't contain more than 50 records for a
        // single player
        let mut pagination = RecordPagination::default();
        pagination.player = Some(player_id);
        let records_before = pagination.page(&mut connection).await.unwrap();

        let patch = PatchPlayer {
            banned: Some(true),
            ..Default::default()
        };

        let patched_player = player.apply_patch(patch, &mut connection).await;

        assert!(patched_player.is_ok(), "{:?}", patched_player.unwrap_err());

        let patched_player = patched_player.unwrap();

        // Test data had the player with 3 records, one of them submitted
        // the submission should have been deleted, the other
        assert!(patched_player.records.is_empty(), "{:?}", patched_player);

        // see if database and model are consistent
        let player = Player::by_id(player_id, &mut connection).await.unwrap();
        let player = player.upgrade(&mut connection).await.unwrap();

        assert_eq!(player, patched_player);

        // Check if all records where properly updated
        let records_after = pagination.page(&mut connection).await.unwrap();

        assert_eq!(records_after.len(), 2);

        // all records must be rejected
        for record in records_after {
            assert_eq!(record.status, RecordStatus::Rejected);
        }
    }
}
