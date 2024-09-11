use crate::{
    error::{DemonlistError, Result},
    nationality::Nationality,
    player::{claim::PlayerClaim, DatabasePlayer, FullPlayer, Player},
    record::{approved_records_by, FullRecord},
};
use log::info;
use pointercrate_core::util::{non_nullable, nullable};
use serde::Deserialize;
use sqlx::PgConnection;

#[derive(Debug, Deserialize, Default)]
pub struct PatchPlayer {
    #[serde(default, deserialize_with = "non_nullable")]
    pub name: Option<String>,

    #[serde(default, deserialize_with = "non_nullable")]
    pub banned: Option<bool>,

    #[serde(default, deserialize_with = "nullable")]
    pub nationality: Option<Option<String>>,

    #[serde(default, deserialize_with = "nullable")]
    pub subdivision: Option<Option<String>>,
}

impl FullPlayer {
    pub async fn apply_patch(mut self, patch: PatchPlayer, connection: &mut PgConnection) -> Result<Self> {
        let mut new_nationality = match patch.nationality {
            None => self.player.nationality.clone(),
            Some(None) => None,
            Some(Some(ref code_or_name)) => Some(Nationality::by_country_code_or_name(code_or_name, connection).await?),
        };

        match new_nationality {
            Some(ref mut nationality) => {
                nationality.subdivision = match patch.subdivision {
                    None => self
                        .player
                        .nationality
                        .as_ref()
                        .filter(|n| n.same_country_as(nationality))
                        .map(|n| n.subdivision.clone())
                        .unwrap_or(None),
                    Some(None) => None,
                    Some(Some(subdivision_code)) => Some(nationality.subdivision_by_code(&subdivision_code, connection).await?),
                }
            },
            None => {
                if matches!(patch.subdivision, Some(Some(_))) {
                    return Err(DemonlistError::NoNationSet);
                }
            },
        }

        if new_nationality != self.player.nationality {
            self.player.set_nationality(new_nationality, connection).await?;
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

        self.player.score = self.player.base.update_score(connection).await?;

        Ok(self)
    }

    pub async fn set_name(&mut self, name: String, connection: &mut PgConnection) -> Result<()> {
        let name = name.trim().to_string();

        // Nothing to be done
        if name == self.player.base.name.as_ref() {
            return Ok(());
        } else if name.to_lowercase() != self.player.base.name.to_lowercase() {
            // If they are equal case insensitively, we're only doing a cosmetic rename, which won't
            // even require a merge

            // try to see if a player with new name already exists
            match DatabasePlayer::by_name(name.as_ref(), &mut *connection).await {
                Ok(existing) => self.merge(existing, &mut *connection).await?,
                Err(DemonlistError::PlayerNotFoundName { .. }) => (),
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

        let claim_on_self = PlayerClaim::verified_claim_on(self.player.base.id, &mut *connection).await?;
        let claim_on_with = PlayerClaim::verified_claim_on(with.id, &mut *connection).await?;

        match (claim_on_self, claim_on_with) {
            (Some(_), Some(_)) => {
                return Err(DemonlistError::ConflictingClaims {
                    player1: self.player.base.name.clone(),
                    player2: with.name.clone(),
                })
            },
            (Some(_), None) => {
                sqlx::query!("DELETE FROM player_claims WHERE player_id = $1", with.id)
                    .execute(&mut *connection)
                    .await?;
            },
            (None, Some(_)) => {
                sqlx::query!("DELETE FROM player_claims WHERE player_id = $1", self.player.base.id)
                    .execute(&mut *connection)
                    .await?;
                sqlx::query!(
                    "UPDATE player_claims SET player_id = $1 WHERE player_id = $2",
                    self.player.base.id,
                    with.id
                )
                .execute(&mut *connection)
                .await?;
            },
            (None, None) => {
                sqlx::query!(
                    "UPDATE player_claims SET player_id = $1 WHERE player_id = $2",
                    self.player.base.id,
                    with.id
                )
                .execute(&mut *connection)
                .await?;
            },
        }

        // First, delete duplicate creator entries
        let deleted = sqlx::query!(
            "DELETE FROM creators AS c1 WHERE c1.creator = $2 AND EXISTS (SELECT 1 FROM creators AS c2 WHERE c2.demon = c1.demon AND \
             c2.creator = $1)",
            self.player.base.id,
            with.id
        )
        .execute(&mut *connection)
        .await?;

        info!(
            "Deleted {} duplicate creator entries while merging {} and {}",
            deleted.rows_affected(),
            self,
            with
        );

        // Transfer all other creator entries over
        let updated = sqlx::query!("UPDATE creators SET creator = $1 WHERE creator = $2", self.player.base.id, with.id)
            .execute(&mut *connection)
            .await?;

        info!("Transferred {} creator entries from {} to {}", updated.rows_affected(), with, self);

        // Transfer over verifier and publisher entries

        let updated_verifiers = sqlx::query!("UPDATE demons SET verifier = $1 WHERE verifier = $2", self.player.base.id, with.id)
            .execute(&mut *connection)
            .await?;
        let updated_publishers = sqlx::query!(
            "UPDATE demons SET publisher = $1 WHERE publisher = $2",
            self.player.base.id,
            with.id
        )
        .execute(&mut *connection)
        .await?;

        info!(
            "Transferred over {} verifier and {} publisher entires from {} to {}",
            updated_verifiers.rows_affected(),
            updated_publishers.rows_affected(),
            with,
            self
        );

        // Alright so merging records is HARD. We already implemented it over in the record patching, so
        // while somewhat inefficient maybe, we'll just call that code for each record of the current player
        for row in sqlx::query!("SELECT id FROM records WHERE player = $1", with.id)
            .fetch_all(&mut *connection)
            .await?
        {
            // FIXME: this is really inefficient and can be made a lot faster by simple moving around some code
            // in the FullRecord impls
            let mut record = FullRecord::by_id(row.id, &mut *connection).await?;
            info!("Moving record {} over to new player {}", record, self.player.base);
            record.set_player(self.player.base.clone(), &mut *connection).await?
        }

        self.records = approved_records_by(&self.player.base, &mut *connection).await?;

        // Transfer all records over, now that they're unique
        let updated = sqlx::query!("UPDATE records SET player = $1 WHERE player = $2", self.player.base.id, with.id)
            .execute(&mut *connection)
            .await?;

        info!("Moved {} records from {} to {}", updated.rows_affected(), with, self);

        // Delete the second player
        sqlx::query!("DELETE FROM players WHERE id = $1", with.id)
            .execute(connection)
            .await?;

        Ok(())
    }
}

impl Player {
    pub async fn set_nationality(&mut self, nationality: Option<Nationality>, connection: &mut PgConnection) -> Result<()> {
        let iso_country_code = nationality.as_ref().map(|n| &n.iso_country_code);
        let subdivision_code = nationality.as_ref().map(|n| n.subdivision.as_ref().map(|s| &s.iso_code)).flatten();

        sqlx::query!(
            "UPDATE players SET nationality = $1, subdivision = $2 WHERE id = $3",
            iso_country_code,
            subdivision_code,
            self.base.id
        )
        .execute(&mut *connection)
        .await?;

        if let Some(ref mut nationality) = self.nationality {
            nationality.update_nation_score(connection).await?;
        }
        self.nationality = nationality;

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
        .execute(&mut *connection)
        .await?;

        info!("Deleted {} submissions while banning {}", deleted.rows_affected(), self);

        // We can simply reject all accepted records here! All submitted records were deleted above, and we
        // don't have to worry about conflicts with existing rejected record when setting status to
        // 'rejected' since rejected records are globally unique!

        // Now, reject all previously accepted records
        let updated = sqlx::query!("UPDATE records SET status_ = 'REJECTED' WHERE player = $1", self.id)
            .execute(&mut *connection)
            .await?;

        info!("Rejected {} records while banning {}", updated.rows_affected(), self);

        // Actually ban the player
        sqlx::query!("UPDATE players SET banned = true WHERE id = $1", self.id)
            .execute(connection)
            .await?;

        self.banned = true;

        Ok(())
    }
}
