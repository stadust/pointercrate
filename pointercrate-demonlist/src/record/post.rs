use crate::{
    demon::MinimalDemon,
    error::{DemonlistError, Result},
    player::{claim::PlayerClaim, DatabasePlayer},
    record::{FullRecord, RecordStatus},
    submitter::Submitter,
};
use derive_more::Display;
use log::debug;
use serde::Deserialize;
use sqlx::PgConnection;
use url::Url;

#[derive(Deserialize, Debug, Display)]
#[display(fmt = "{}% on {} by {} [status: {}]", progress, demon, player, status)]
pub struct Submission {
    progress: i16,
    player: String,
    demon: i32,
    #[serde(default)]
    video: Option<String>,
    #[serde(default)]
    raw_footage: Option<String>,
    #[serde(default)]
    status: RecordStatus,

    /// An initial, submitter provided note for the demon.
    #[serde(default)]
    note: Option<String>,
}

#[derive(Debug)]
pub struct NormalizedSubmission {
    progress: i16,
    player: DatabasePlayer,
    demon: MinimalDemon,
    status: RecordStatus,

    video: Option<String>,
    raw_footage: Option<String>,
    note: Option<String>,
}

#[derive(Debug)]
pub struct ValidatedSubmission {
    progress: i16,
    video: Option<String>,
    raw_footage: Option<String>,
    status: RecordStatus,
    player: DatabasePlayer,
    demon: MinimalDemon,
    note: Option<String>,
}

impl Submission {
    pub fn has_video(&self) -> bool {
        self.video.is_some()
    }

    pub fn status(&self) -> RecordStatus {
        self.status
    }

    pub async fn normalize(self, connection: &mut PgConnection) -> Result<NormalizedSubmission> {
        // validate video
        let video = match self.video {
            Some(ref video) => Some(crate::video::validate(video)?),
            None => None,
        };

        // Resolve player and demon name against the database
        let player = DatabasePlayer::by_name_or_create(self.player.as_ref(), connection).await?;
        let demon = MinimalDemon::by_id(self.demon, connection).await?;

        Ok(NormalizedSubmission {
            progress: self.progress,
            player,
            demon,
            status: self.status,
            video,
            raw_footage: self.raw_footage,
            note: self.note,
        })
    }
}

impl NormalizedSubmission {
    pub async fn verified_player_claim(&self, connection: &mut PgConnection) -> Result<Option<PlayerClaim>> {
        PlayerClaim::verified_claim_on(self.player.id, connection).await
    }

    pub async fn validate(self, connection: &mut PgConnection) -> Result<ValidatedSubmission> {
        // Banned player can't have records on the list
        if self.player.banned {
            return Err(DemonlistError::PlayerBanned);
        }

        // Cannot submit records for the legacy list (it is possible to directly add them for list mods)
        if self.demon.position > crate::config::extended_list_size() && self.status == RecordStatus::Submitted {
            return Err(DemonlistError::SubmitLegacy);
        }

        // Can only submit 100% records for the extended list (it is possible to directly add them for list
        // mods)
        if self.demon.position > crate::config::list_size() && self.progress != 100 && self.status == RecordStatus::Submitted {
            return Err(DemonlistError::Non100Extended);
        }

        let requirement = self.demon.requirement(&mut *connection).await?;

        // Check if the record meets the record requirement for this demon
        if self.progress > 100 || self.progress < requirement {
            return Err(DemonlistError::InvalidProgress { requirement });
        }

        debug!("Submission is valid, checking for duplicates!");

        // Search for existing records. If a video exists, we also check if a record with
        // exactly that video exists.

        if let Some(ref video) = self.video {
            if let Some(row) = sqlx::query!(r#"SELECT id, status_::text as "status_!: String" FROM records WHERE video = $1"#, video.to_string())
                .fetch_optional(&mut *connection) // FIXME(sqlx)
                .await?
            {
                return Err(DemonlistError::SubmissionExists {
                    existing: row.id,
                    status: RecordStatus::from_sql(&row.status_),
                });
            }
        }

        let existing = sqlx::query!(
            r#"SELECT id, status_::text as "status_!: String" FROM records WHERE demon = $1 AND player = $2 AND (status_ = 'REJECTED' OR status_ = 
             'UNDER_CONSIDERATION' OR (status_ = 'APPROVED' AND progress >= $3)) LIMIT 1"#,
            self.demon.id,
            self.player.id,
            self.progress
        )
            .fetch_optional(&mut *connection)
            .await?;

        if let Some(row) = existing {
            return Err(DemonlistError::SubmissionExists {
                existing: row.id,
                status: RecordStatus::from_sql(&row.status_),
            });
        }

        match self.raw_footage {
            Some(ref raw) => {
                let _ = Url::parse(raw).map_err(|_| DemonlistError::MalformedRawUrl)?;
            },
            None if self.status == RecordStatus::Submitted => {
                // list mods can submit without raw
                return Err(DemonlistError::RawRequired);
            },
            _ => (),
        }

        Ok(ValidatedSubmission {
            progress: self.progress,
            video: self.video,
            raw_footage: self.raw_footage,
            status: self.status,
            player: self.player,
            demon: self.demon,
            note: self.note,
        })
    }
}

impl ValidatedSubmission {
    pub async fn create(self, submitter: Submitter, connection: &mut PgConnection) -> Result<FullRecord> {
        let id = sqlx::query!(
            "INSERT INTO records (progress, video, status_, player, submitter, demon, raw_footage) VALUES ($1, $2::TEXT, 'SUBMITTED', $3, $4, $5, $6) RETURNING id",
            self.progress,
            self.video,
            self.player.id,
            submitter.id,
            self.demon.id,
            self.raw_footage
        )
        .fetch_one(&mut *connection)
        .await?
        .id;

        let mut record = FullRecord {
            id,
            progress: self.progress,
            video: self.video,
            raw_footage: self.raw_footage,
            status: RecordStatus::Submitted,
            player: self.player,
            demon: self.demon,
            submitter: Some(submitter),
        };

        // Dealing with different status and upholding their invariant is complicated, we should not
        // duplicate that code!
        if self.status != RecordStatus::Submitted {
            record.set_status(self.status, &mut *connection).await?;
        }

        if let Some(note) = self.note {
            if !note.trim().is_empty() {
                sqlx::query!("INSERT INTO record_notes (record, content) VALUES ($1, $2)", record.id, note)
                    .execute(&mut *connection)
                    .await?;
            }
        }

        if self.status != RecordStatus::Submitted {
            record.player.update_score(connection).await?;
        }

        Ok(record)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        demon::MinimalDemon,
        error::DemonlistError,
        player::DatabasePlayer,
        record::{post::NormalizedSubmission, RecordStatus},
    };
    use sqlx::{pool::PoolConnection, Postgres};

    #[sqlx::test(migrations = "../migrations")]
    async fn test_banned_cannot_submit(mut conn: PoolConnection<Postgres>) {
        let result = NormalizedSubmission {
            progress: 100,
            player: DatabasePlayer {
                id: 1,
                name: "stardust1971".to_string(),
                banned: true,
            },
            demon: MinimalDemon {
                id: 1,
                position: 1,
                name: "Bloodbath".to_string(),
            },
            status: RecordStatus::Submitted,
            video: None,
            raw_footage: None,
            note: None,
        }
        .validate(&mut conn)
        .await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), DemonlistError::PlayerBanned)
    }
}
