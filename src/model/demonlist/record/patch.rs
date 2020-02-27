use crate::{
    cistring::CiString,
    error::PointercrateError,
    model::demonlist::{
        demon::MinimalDemon,
        player::DatabasePlayer,
        record::{FullRecord, RecordStatus},
    },
    util::{non_nullable, nullable},
    Result,
};
use log::info;
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
    player: Option<CiString>,

    #[serde(default, deserialize_with = "non_nullable")]
    demon: Option<CiString>,

    #[serde(default, deserialize_with = "non_nullable")]
    demon_id: Option<i32>,

    #[serde(default, deserialize_with = "nullable")]
    notes: Option<Option<String>>,
}

impl FullRecord {
    /// Must be called inside a transaction
    pub async fn apply_patch(mut self, data: PatchRecord, connection: &mut PgConnection) -> Result<Self> {
        // Do notes first so that notes copied during further modifications don't get overriden!
        if let Some(notes) = data.notes {
            match notes {
                None => self.delete_notes(connection).await?,
                Some(notes) => self.set_notes(notes, connection).await?,
            }
        }

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
            (Some(_), Some(_)) => return Err(PointercrateError::MutuallyExclusive),
            _ => (),
        }

        Ok(self)
    }

    /// Function for handling the case where after updating the record with `record_id` it would
    /// violate the UNIQUE (player, demon, status) constraint.
    ///
    /// This can happen when the player, demon or status is updated. This function assumes that 2
    /// out of `player`, `demon` and `status` match the values of the given record, with the
    /// non-matching one set to what the record should be updated to.
    /// If a record with the given (player, demon, status) combo exists, it is deleted. If it has
    /// higher progress than the given record, the given record will have its progress and video
    /// set to that of the deleted record. Notes are merged, as always.
    ///
    /// This method does not set any of the (player, demon, status) fields.
    async fn handle_potential_duplicate(
        &mut self, player: &DatabasePlayer, demon: &MinimalDemon, status: RecordStatus, connection: &mut PgConnection,
    ) -> Result<()> {
        struct Existing {
            progress: i16,
            video: Option<String>,
            notes: Option<String>,
        }

        let existing = sqlx::query_as!(
            Existing,
            "DELETE FROM records WHERE player = $1 AND demon = $2 AND status_ = cast($3::text as record_status) RETURNING progress, \
             video::text, notes", /* FIXME(sqlx) once custom enums are
                                   * supported */
            player.id,
            demon.id,
            status.to_string()
        )
        .fetch_optional(connection)
        .await?;

        if let Some(row) = existing {
            info!(
                "Updating the record {} to be ({},{},{}) caused a duplicate, which had progress {}",
                self, player, demon, status, row.progress
            );

            if row.progress > self.progress {
                // FIXME(sqlx) Option types
                match row.video {
                    None =>
                        sqlx::query!(
                            "UPDATE records SET progress = $1, video = NULL WHERE id = $2",
                            row.progress,
                            self.id
                        )
                        .execute(connection)
                        .await?,
                    Some(ref video) =>
                        sqlx::query!(
                            "UPDATE records SET progress = $1, video = $2::text WHERE id = $3",
                            row.progress,
                            video.to_string(),
                            self.id
                        )
                        .execute(connection)
                        .await?,
                };

                self.video = row.video;
                self.progress = row.progress;
            }

            if let Some(additional_notes) = row.notes {
                self.append_notes(additional_notes, connection).await?;
            }
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
        sqlx::query!("UPDATE records SET video = $1::text WHERE id = $2", video, self.id)
            .execute(connection)
            .await?;

        self.video = Some(video);

        Ok(())
    }

    pub async fn set_demon(&mut self, demon: MinimalDemon, connection: &mut PgConnection) -> Result<()> {
        let requirement = demon.requirement(connection).await?;

        if self.progress < requirement {
            return Err(PointercrateError::InvalidProgress { requirement })?
        }

        self.handle_potential_duplicate(&self.player.clone(), &demon, self.status, connection)// FIXME: proper borrowing
            .await?;

        sqlx::query!("UPDATE records SET demon = $1 WHERE id = $2", demon.id, self.id)
            .execute(connection)
            .await?;

        self.demon = demon;

        Ok(())
    }

    pub async fn set_player(&mut self, player: DatabasePlayer, connection: &mut PgConnection) -> Result<()> {
        self.handle_potential_duplicate(&player, &self.demon.clone(), self.status, connection) // FIXME: proper borrowing
            .await?;

        sqlx::query!("UPDATE records SET player = $1 WHERE id = $2", player.id, self.id)
            .execute(connection)
            .await?;

        self.player = player;

        Ok(())
    }

    pub async fn set_status(&mut self, status: RecordStatus, connection: &mut PgConnection) -> Result<()> {
        self.handle_potential_duplicate(&self.player.clone(), &self.demon.clone(), status, connection) // FIXME: proper borrowing
            .await?;

        sqlx::query!(
            "UPDATE records SET status_ = cast($1::text as record_status) WHERE id = $2", /* FIXME(sqlx) ridiculous query
                                                                                           * format to trick sqlx into working
                                                                                           * with custom types */
            status.to_string(),
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
    /// demon)-tuple are deleted.
    pub async fn set_progress(&mut self, progress: i16, connection: &mut PgConnection) -> Result<()> {
        let requirement = self.demon.requirement(connection).await?;

        if progress > 100 || progress < requirement {
            return Err(PointercrateError::InvalidProgress { requirement })?
        }

        if self.status == RecordStatus::Approved {
            struct Note {
                notes: Option<String>,
            }

            let notes_to_transfer = sqlx::query_as!(
                Note,
                "DELETE FROM records WHERE player = $1 AND demon = $2 AND status_='SUBMITTED' RETURNING notes",
                self.player.id,
                self.demon.id
            )
            .fetch_all(connection)
            .await?;

            info!(
                "Changing progress of record {} from {} to {} caused the deletion of {} submissions",
                self,
                self.progress,
                progress,
                notes_to_transfer.len()
            );

            self.append_notes(
                notes_to_transfer
                    .into_iter()
                    .filter_map(|row| row.notes)
                    .collect::<Vec<_>>()
                    .join("\n"),
                connection,
            )
            .await?;
        }

        sqlx::query!("UPDATE records SET progress = $1 WHERE id = $2", progress, self.id)
            .execute(connection)
            .await?;

        self.progress = progress;

        Ok(())
    }

    pub async fn delete_notes(&mut self, connection: &mut PgConnection) -> Result<()> {
        sqlx::query!("UPDATE records SET notes = NULL WHERE id = $1", self.id)
            .execute(connection)
            .await?;

        self.notes = None;

        Ok(())
    }

    pub async fn set_notes(&mut self, notes: String, connection: &mut PgConnection) -> Result<()> {
        sqlx::query!("UPDATE records SET notes = $2 WHERE id = $1", self.id, notes)
            .execute(connection)
            .await?;

        self.notes = Some(notes);

        Ok(())
    }

    pub async fn append_notes(&mut self, to_append: String, connection: &mut PgConnection) -> Result<()> {
        match self.notes {
            None => self.set_notes(to_append, connection).await?,
            Some(ref mut existing) => {
                sqlx::query!("UPDATE records SET notes = notes || $2 WHERE id = $1", self.id, to_append)
                    .execute(connection)
                    .await?;

                existing.push_str(&to_append)
            },
        }

        Ok(())
    }
}
// impl Patch<PatchRecord> for FullRecord {
// fn patch(mut self, mut patch: PatchRecord, ctx: RequestContext) -> Result<Self> {
// ctx.check_permissions(perms!(ListHelper or ListModerator or ListAdministrator))?;
// ctx.check_if_match(&self)?;
//
// FIXME: This needs to do the whole "locate duplicate, compare, delete" dance
//
// let connection = ctx.connection();
//
// info!("Patching record {} with {}", self, patch);
//
// validate_nullable!(patch: Record::validate_video[video]);
//
// let demon = Demon::get(
// match patch.demon {
// None => self.demon.name.as_ref(),
// Some(ref demon) => demon.as_ref(),
// },
// ctx,
// )?;
// let progress = patch.progress.unwrap_or(self.progress);
//
// if progress > 100 || progress < demon.requirement {
// return Err(PointercrateError::InvalidProgress {
// requirement: demon.requirement,
// })?
// }
//
// let map = move |_| {
// MinimalDemon {
// id: demon.id,
// name: demon.name,
// position: demon.position,
// }
// };
// let map2 = |name: &CiStr| DatabasePlayer::get(name, ctx);
//
// map_patch!(self, patch: map => demon);
// try_map_patch!(self, patch: map2 => player);
// patch!(self, patch: progress, video, status, notes);
//
// connection.transaction(move || {
// If there is a record that would validate the unique (status_, demon, player),
// with higher progress than this one, this query would find it
// let max_progress: Option<i16> = records::table
// .select(records::all_columns)
// .filter(records::player.eq(&self.player.id))
// .filter(records::demon.eq(&self.demon.id))
// .filter(records::status_.eq(&self.status))
// .filter(records::id.ne(&self.id))
// .select(diesel::dsl::max(records::progress))
// .get_result::<Option<i16>>(connection)?;
//
// if let Some(max_progress) = max_progress {
// if max_progress > self.progress {
// We simply make `self` the same as that record, causing it to later get
// deleted
// let record = DatabaseRecord::all()
// .filter(records::player.eq(&self.player.id))
// .filter(records::demon.eq(&self.demon.id))
// .filter(records::status_.eq(&self.status))
// .filter(records::progress.eq(&max_progress))
// .get_result::<DatabaseRecord>(connection)?;
//
// self.video = record.video;
// self.progress = record.progress;
// }
// }
//
// By now, our record is for sure the one with the highest progress - all others can be
// deleted
// diesel::delete(
// records::table
// .filter(records::player.eq(self.player.id))
// .filter(records::demon.eq(self.demon.id))
// .filter(records::status_.eq(RecordStatus::Approved).or(records::status_.eq(self.status)))
// .filter(records::progress.le(self.progress))
// .filter(records::id.ne(self.id)),
// )
// .execute(connection)?;
//
// diesel::update(records::table)
// .filter(records::id.eq(&self.id))
// .set((
// records::progress.eq(&self.progress),
// records::video.eq(&self.video),
// records::status_.eq(&self.status),
// records::player.eq(&self.player.id),
// records::demon.eq(&self.demon.id),
// records::notes.eq(&self.notes),
// ))
// .execute(connection)?;
//
// Ok(self)
// })
// }
// }
