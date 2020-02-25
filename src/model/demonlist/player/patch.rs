use crate::{
    cistring::{CiStr, CiString},
    error::PointercrateError,
    model::{
        demonlist::player::{DatabasePlayer, FullPlayer, Player},
        nationality::Nationality,
    },
    util::{non_nullable, nullable},
    Result,
};
use log::info;
use serde::Deserialize;
use sqlx::PgConnection;

#[derive(Debug, Deserialize)]
pub struct PatchPlayer {
    #[serde(default, deserialize_with = "non_nullable")]
    name: Option<CiString>,

    #[serde(default, deserialize_with = "non_nullable")]
    banned: Option<bool>,

    #[serde(default, deserialize_with = "nullable")]
    nationality: Option<Option<CiString>>,
}

impl FullPlayer {
    pub async fn apply_patch(self, patch: PatchPlayer, connection: &mut PgConnection) -> Result<Self> {
        Ok(FullPlayer {
            player: self.player.apply_patch(patch, connection).await?,
            ..self
        })
    }
}

impl Player {
    pub async fn apply_patch(mut self, patch: PatchPlayer, connection: &mut PgConnection) -> Result<Self> {
        if let Some(nationality) = patch.nationality {
            match nationality {
                Some(ident) =>
                    self.set_nationality(Nationality::by_country_code_or_name(ident.as_ref(), connection).await?, connection)
                        .await?,
                None => self.reset_nationality(connection).await?,
            }
        }

        if let Some(banned) = patch.banned {
            if banned && !self.base.banned {
                self.base.ban(connection).await?;
            } else if !banned && self.base.banned {
                self.base.unban(connection).await?;
            }
        }

        Ok(self)
    }

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
    pub async fn set_name(&mut self, name: &CiStr, connection: &mut PgConnection) -> Result<()> {
        if name == &self.name {
            return Ok(())
        }

        // try to see if a player with new name already exists
        match DatabasePlayer::by_name(name, connection).await {
            Ok(existing) => self.merge(existing, connection).await?,
            Err(PointercrateError::ModelNotFound { .. }) => (),
            Err(err) => return Err(err),
        }

        sqlx::query!("UPDATE players SET name = $1::text WHERE id = $2", name.to_string(), self.id)
            .execute(connection)
            .await?;

        Ok(())
    }

    pub async fn unban(&mut self, connection: &mut PgConnection) -> Result<()> {
        sqlx::query!("UPDATE players SET banned = false WHERE id=$1", self.id)
            .execute(connection)
            .await?;

        Ok(())
    }

    pub async fn ban(&mut self, connection: &mut PgConnection) -> Result<()> {
        // Delete all submissions for this player
        let deleted = sqlx::query!("DELETE FROM records WHERE player = $1 AND status_ = 'SUBMITTED'", self.id)
            .execute(connection)
            .await?;

        info!("Deleted {} submissions while banning {}", deleted, self);

        // FIXME: figure out where the constraint mentioned below went

        // At this point the player only has two kinds of records anymore: accepted and rejected.
        // We need to reject all accepted records here. In theory, there should be a UNIQUE (player, demon,
        // status) constraint on the database, so we need to make sure that we delete rejected records of
        // this player on demons they also have an accepted record on.

        let new_notes = sqlx::query_file!("sql/prepare_player_ban.sql", self.id)
            .fetch_all(connection)
            .await?;

        // Transfer notes of deleted records (also see DatabasePlayer::merge)

        for new_note in new_notes {
            if !new_note.notes.is_empty() {
                sqlx::query!(
                    "UPDATE records SET notes = notes || '\n' || $1 WHERE id = $2",
                    new_note.notes,
                    new_note.id
                )
                .execute(connection)
                .await?;
            }
        }

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

    /// Merges the given player into `Self`, deleting `with`.
    ///
    /// Note that this method **does not** rename `Self`
    pub async fn merge(&mut self, with: DatabasePlayer, connection: &mut PgConnection) -> Result<()> {
        // First, delete duplicate creator entries

        let deleted = sqlx::query!(
            "DELETE FROM creators AS c1 WHERE c1.creator = $2 AND EXISTS (SELECT 1 FROM creators AS c2 WHERE c2.demon = c1.demon AND \
             c2.creator = $1)",
            self.id,
            with.id
        )
        .execute(connection)
        .await?;

        info!("Deleted {} duplicate creator entries while merging {} and {}", deleted, self, with);

        // Transfer all other creator entries over
        let updated = sqlx::query!("UPDATE creators SET creator = $1 WHERE creator = $2", self.id, with.id)
            .execute(connection)
            .await?;

        info!("Transferred {} creator entries from {} to {}", updated, with, self);

        // Transfer over verifier and publisher entries

        let updated_verifiers = sqlx::query!("UPDATE demons SET verifier = $1 WHERE verifier = $2", self.id, with.id)
            .execute(connection)
            .await?;
        let updated_publishers = sqlx::query!("UPDATE demons SET publisher = $1 WHERE publisher = $2", self.id, with.id)
            .execute(connection)
            .await?;

        info!(
            "Transferred over {} verifier and {} publisher entires from {} to {}",
            updated_verifiers, updated_publishers, with, self
        );

        // delete all records that would be duplicated in the merge
        // If the players both have a record of the same state on a demon, the record with lower progress
        // will be deleted. If both have equal progress, the one with the smaller id will be deleted.
        // Since each deleted record will have a "witness" for deletion, this witness will get the notes of
        // the deleted record added to it.
        let new_notes = sqlx::query_file!("sql/prepare_player_merge.sql", self.id, with.id)
            .fetch_all(connection)
            .await?;

        for new_note in new_notes {
            if !new_note.notes.is_empty() {
                sqlx::query!(
                    "UPDATE records SET notes = notes || '\n' || $1 WHERE id = $2",
                    new_note.notes,
                    new_note.id
                )
                .execute(connection)
                .await?;
            }
        }

        // Transfer all records over, now that they're unique
        let updated = sqlx::query!("UPDATE records SET player = $1 WHERE player = $2", self.id, with.id)
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
