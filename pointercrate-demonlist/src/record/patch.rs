use crate::{
    demon::MinimalDemon,
    error::{DemonlistError, Result},
    player::DatabasePlayer,
    record::{FullRecord, RecordStatus},
};
use log::{info, warn};
use pointercrate_core::{
    error::CoreError,
    util::{non_nullable, nullable},
};
use serde::Deserialize;
use sqlx::PgConnection;

#[derive(Debug, Deserialize)]
pub struct PatchRecord {
    #[serde(default, deserialize_with = "non_nullable")]
    progress: Option<i16>,

    #[serde(default, deserialize_with = "nullable")]
    video: Option<Option<String>>,

    #[serde(default, deserialize_with = "non_nullable")]
    status: Option<RecordStatus>,

    #[serde(default, deserialize_with = "non_nullable")]
    player: Option<String>,

    #[serde(default, deserialize_with = "non_nullable")]
    demon: Option<String>,

    #[serde(default, deserialize_with = "non_nullable")]
    demon_id: Option<i32>,
}

impl FullRecord {
    /// Must be called inside a transaction
    pub async fn apply_patch(mut self, data: PatchRecord, connection: &mut PgConnection) -> Result<Self> {
        info!("Applying patch {:?} for record {}", data, self);

        if let Some(progress) = data.progress {
            self.set_progress(progress, connection).await?;
        }

        if let Some(video) = data.video {
            match video {
                None => self.delete_video(connection).await?,
                Some(video) => self.set_video(video, connection).await?,
            }
        }

        if let Some(status) = data.status {
            self.set_status(status, connection).await?
        }

        if let Some(player) = data.player {
            let player = DatabasePlayer::by_name_or_create(player.as_ref(), connection).await?;

            self.set_player(player, connection).await?;
        }

        match (data.demon, data.demon_id) {
            (Some(demon_name), None) =>
                self.set_demon(MinimalDemon::by_name(demon_name.as_ref(), connection).await?, connection)
                    .await?,
            (None, Some(demon_id)) => self.set_demon(MinimalDemon::by_id(demon_id, connection).await?, connection).await?,
            (Some(_), Some(_)) => Err(CoreError::MutuallyExclusive)?,
            _ => (),
        }

        Ok(self)
    }

    /// Prepared turning `self` into a (player, demon)-record (either player or demon will be
    /// changed)
    async fn ensure_invariants(&mut self, player: i32, demon: i32, connection: &mut PgConnection) -> Result<()> {
        if self.player.id == player && self.demon.id == demon {
            warn!("Record::ensure_invariants was called, but the given player and demon ids match those we already have. Doing nothing.");

            return Ok(())
        }

        match self.status {
            RecordStatus::Rejected => {
                // The record needs to be globally unique, so delete all (player, demon) records

                let notes_transferred = sqlx::query!(
                    "UPDATE record_notes SET record = $1 FROM records WHERE record_notes.record = records.id AND records.demon = $2 AND \
                     records.player = $3",
                    self.id,
                    demon,
                    player
                )
                .execute(&mut *connection)
                .await?;

                let records_deleted = sqlx::query!("DELETE FROM records WHERE player = $1 AND demon = $2", player, demon)
                    .execute(connection)
                    .await?;

                info!(
                    "Turning {} into a ({}, {})-record caused the transfer of {} notes and the deletion of {} records!",
                    self,
                    player,
                    demon,
                    notes_transferred.rows_affected(),
                    records_deleted.rows_affected()
                );
            },
            RecordStatus::Approved => {
                // In this case we have to do multiple things:
                // * delete all (player, demon)-records that are 'rejected' (at most one) TODO: maybe reconsider?
                // * if a (player, demon)-record exists that is 'approved' and has higher progress than this one, we
                //   override our progress and video with the values of that record
                // * delete all (player, demon)-records that are 'submitted' with a progress (potentially as
                //   determined above) less than or equal to that of this record

                struct _Existing {
                    id: i32,
                    progress: i16,
                    video: Option<String>,
                }

                let row = sqlx::query_as!(
                    _Existing,
                    "SELECT id, progress, video::TEXT FROM records WHERE status_ = 'APPROVED' AND demon = $1 AND player = $2 AND progress \
                     > $3",
                    demon,
                    player,
                    self.progress
                )
                .fetch_optional(&mut *connection)
                .await?;

                if let Some(row) = row {
                    sqlx::query!("DELETE FROM records WHERE id = $1", row.id)
                        .execute(&mut *connection)
                        .await?;
                    sqlx::query("UPDATE records SET video = $1::TEXT, progress = $2 WHERE id = $3")
                        .bind(&row.video)
                        .bind(row.progress)
                        .bind(self.id)
                        .execute(&mut *connection)
                        .await?;

                    self.progress = row.progress;
                    self.video = row.video;
                }

                let notes_transferred = sqlx::query!(
                    "UPDATE record_notes SET record = $1 FROM records WHERE record_notes.record = records.id AND records.demon = $2 AND \
                     records.player = $3 AND (records.status_ = 'REJECTED' OR records.progress <= $4)",
                    self.id,
                    demon,
                    player,
                    self.progress
                )
                .execute(&mut *connection)
                .await?;

                let records_deleted = sqlx::query!(
                    "DELETE FROM records WHERE demon = $1 AND player = $2 AND (status_ = 'REJECTED' OR progress <= $3)",
                    demon,
                    player,
                    self.progress
                )
                .execute(connection)
                .await?;

                info!(
                    "Turning {} into a ({}, {})-record caused the transfer of {} notes and the deletion of {} records!",
                    self,
                    player,
                    demon,
                    notes_transferred.rows_affected(),
                    records_deleted.rows_affected()
                );
            },
            // Nothing needed to be done here!
            RecordStatus::Submitted | RecordStatus::UnderConsideration => {},
        }

        Ok(())
    }

    pub async fn delete_video(&mut self, connection: &mut PgConnection) -> Result<()> {
        sqlx::query!("UPDATE records SET video = NULL WHERE id = $1", self.id)
            .execute(connection)
            .await?;

        self.video = None;

        Ok(())
    }

    pub async fn set_video(&mut self, video: String, connection: &mut PgConnection) -> Result<()> {
        let video = crate::video::validate(&video)?;

        if Some(&video) == self.video.as_ref() {
            return Ok(())
        }

        if let Some(row) = sqlx::query!(r#"SELECT id FROM records WHERE video = $1"#, video.to_string())
            .fetch_optional(&mut *connection)
            .await?
        {
            return Err(DemonlistError::DuplicateVideo { id: row.id })
        }

        sqlx::query!("UPDATE records SET video = $1::text WHERE id = $2", video, self.id)
            .execute(connection)
            .await?;

        self.video = Some(video);

        Ok(())
    }

    pub async fn set_demon(&mut self, demon: MinimalDemon, connection: &mut PgConnection) -> Result<()> {
        let requirement = demon.requirement(connection).await?;

        if self.progress < requirement {
            return Err(DemonlistError::InvalidProgress { requirement })
        }

        self.ensure_invariants(self.player.id, self.demon.id, connection).await?;

        sqlx::query!("UPDATE records SET demon = $1 WHERE id = $2", demon.id, self.id)
            .execute(connection)
            .await?;

        self.demon = demon;

        Ok(())
    }

    /// Changes the holder of this record
    ///
    /// If the new player has a record that would stand in conflict with this one, this records
    /// takes precedence and overrides the existing one.
    pub async fn set_player(&mut self, player: DatabasePlayer, connection: &mut PgConnection) -> Result<()> {
        if player.banned && self.status != RecordStatus::Rejected {
            return Err(DemonlistError::PlayerBanned)
        }

        info!("Setting player of record {} to {}", self, player);

        self.ensure_invariants(player.id, self.demon.id, connection).await?;

        sqlx::query!("UPDATE records SET player = $1 WHERE id = $2", player.id, self.id)
            .execute(connection)
            .await?;

        self.player = player;

        Ok(())
    }

    /// Updates this record's status
    pub async fn set_status(&mut self, status: RecordStatus, connection: &mut PgConnection) -> Result<()> {
        // To uphold the invariants outlined in the module documentation, we need to do some preparations.
        // What preparation has to be done, depends on what the current and new status are.
        match (self.status, status) {
            (_, RecordStatus::Rejected) => {
                // if we move a (demon, player)-record to 'rejected', we delete all records for this tuple in other
                // states, to ensure the record will be globally unique after this
                sqlx::query!(
                    "UPDATE record_notes SET record = $1 FROM records WHERE record_notes.record = records.id AND records.player = $2 AND \
                     records.demon = $3",
                    self.id,
                    self.player.id,
                    self.demon.id
                )
                .execute(&mut *connection)
                .await?;

                sqlx::query!(
                    "DELETE FROM records WHERE id <> $1 AND player = $2 AND demon = $3",
                    self.id,
                    self.player.id,
                    self.demon.id
                )
                .execute(&mut *connection)
                .await?;
            },

            // Nothing needed here, approved records are unique while submitted and records under consideration are not
            (RecordStatus::Approved, _) => (),

            // Nothing needed here, a 'rejected' record is globally unique
            (RecordStatus::Rejected, _) => (),

            (RecordStatus::Submitted, RecordStatus::Approved) | (RecordStatus::UnderConsideration, RecordStatus::Approved) => {
                // Since a rejected record is globally unique, we know no other (player,
                // demon)-record is 'rejected'. We also know that the submission has at least as
                // much progress as an 'accepted' (player, demon)-record. We can therefore just
                // delete all other records with less or equal progress to the current one

                sqlx::query!(
                    "UPDATE record_notes SET record = $1 FROM records WHERE record_notes.record = records.id AND records.player = $2 AND \
                     records.demon = $3 AND progress <= $4",
                    self.id,
                    self.player.id,
                    self.demon.id,
                    self.progress
                )
                .execute(&mut *connection)
                .await?;

                sqlx::query!(
                    "DELETE FROM records WHERE id <> $1 AND records.player = $2 AND records.demon = $3 AND progress <= $4",
                    self.id,
                    self.player.id,
                    self.demon.id,
                    self.progress
                )
                .execute(&mut *connection)
                .await?;
            },

            // the other cases just convert back and forth between 'submitted' and 'under consideration', which doesn't change anything
            _ => (),
        }

        sqlx::query!(
            "UPDATE records SET status_ = cast($1::text as record_status) WHERE id = $2", /* FIXME(sqlx) ridiculous query
                                                                                           * format to trick sqlx into working
                                                                                           * with custom types */
            status.to_sql().to_string(),
            self.id
        )
        .execute(connection)
        .await?;

        self.status = status;

        Ok(())
    }

    /// Updates this record's progress
    ///
    /// If this record is approved, all submissions with lower progress of the same (player,
    /// demon)-tuple are deleted and have their notes transferred to this record.
    pub async fn set_progress(&mut self, progress: i16, connection: &mut PgConnection) -> Result<()> {
        let requirement = self.demon.requirement(&mut *connection).await?;

        if progress > 100 || progress < requirement {
            return Err(DemonlistError::InvalidProgress { requirement })
        }

        if self.status == RecordStatus::Approved {
            // Transfer over all notes from the records deleted below
            sqlx::query!(
                "UPDATE record_notes SET record = $1 FROM records WHERE record_notes.record = records.id AND player = $2 AND demon = $3 \
                 AND progress < $4 AND status_='SUBMITTED'",
                self.id,
                self.player.id,
                self.demon.id,
                progress
            )
            .execute(&mut *connection)
            .await?;

            let deleted = sqlx::query!(
                "DELETE FROM records WHERE player = $1 AND demon = $2 AND status_='SUBMITTED'",
                self.player.id,
                self.demon.id
            )
            .execute(&mut *connection)
            .await?;

            info!(
                "Changing progress of record {} from {} to {} caused the deletion of {} submissions",
                self,
                self.progress,
                progress,
                deleted.rows_affected()
            );
        }

        sqlx::query!("UPDATE records SET progress = $1 WHERE id = $2", progress, self.id)
            .execute(connection)
            .await?;

        self.progress = progress;

        Ok(())
    }
}
