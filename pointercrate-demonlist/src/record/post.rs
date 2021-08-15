use crate::{
    demon::MinimalDemon,
    error::{DemonlistError, Result},
    player::DatabasePlayer,
    record::{note::Note, FullRecord, RecordStatus},
    submitter::Submitter,
};
use derive_more::Display;
use log::{debug, info};
use serde::Deserialize;
use sqlx::{PgConnection, Row};

#[derive(Deserialize, Debug, Display)]
#[display(fmt = "{}% on {} by {} [status: {}]", progress, demon, player, status)]
pub struct Submission {
    pub progress: i16,
    pub player: String,
    pub demon: i32,
    #[serde(default)]
    pub video: Option<String>,
    #[serde(default)]
    pub status: RecordStatus,

    /// An initial, submitter provided note for the demon.
    #[serde(default)]
    pub note: Option<String>,
}

pub struct ValidatedSubmission {
    progress: i16,
    video: Option<String>,
    status: RecordStatus,
    player: DatabasePlayer,
    demon: MinimalDemon,
    submitter: Submitter,
    note: Option<String>,
}

impl Submission {
    pub async fn validate(self, submitter: Submitter, connection: &mut PgConnection) -> Result<ValidatedSubmission> {
        info!("Processing record addition '{}' by {}", self, submitter);

        // Banned submitters cannot submit records
        if submitter.banned {
            return Err(DemonlistError::BannedFromSubmissions)
        }

        // validate video
        let video = match self.video {
            Some(ref video) => Some(crate::video::validate(video)?),
            None => None,
        };

        // Resolve player and demon name against the database
        let player = DatabasePlayer::by_name_or_create(self.player.as_ref(), connection).await?;
        // TODO: handle the ambiguous case
        let demon = MinimalDemon::by_id(self.demon, connection).await?;

        // Banned player can't have records on the list
        if player.banned {
            return Err(DemonlistError::PlayerBanned)
        }

        // Cannot submit records for the legacy list (it is possible to directly add them for list mods)
        if demon.position > crate::config::extended_list_size() && self.status == RecordStatus::Submitted {
            return Err(DemonlistError::SubmitLegacy)
        }

        // Can only submit 100% records for the extended list (it is possible to directly add them for list
        // mods)
        if demon.position > crate::config::list_size() && self.progress != 100 && self.status == RecordStatus::Submitted {
            return Err(DemonlistError::Non100Extended)
        }

        let requirement = demon.requirement(&mut *connection).await?;

        // Check if the record meets the record requirement for this demon
        if self.progress > 100 || self.progress < requirement {
            return Err(DemonlistError::InvalidProgress { requirement })
        }

        debug!("Submission is valid, checking for duplicates!");

        // Search for existing records. If a video exists, we also check if a record with
        // exactly that video exists.

        if let Some(ref video) = video {
            if let Some(row) = sqlx::query!(r#"SELECT id, status_::text as "status_!: String" FROM records WHERE video = $1"#, video.to_string())
                .fetch_optional(&mut *connection) // FIXME(sqlx)
                .await?
            {
                return Err(DemonlistError::SubmissionExists {
                    existing: row.id,
                    status: RecordStatus::from_sql(&row.status_),
                })
            }
        }

        let existing = sqlx::query!(
            r#"SELECT id, status_::text as "status_!: String" FROM records WHERE demon = $1 AND player = $2 AND (status_ = 'REJECTED' OR status_ = 
             'UNDER_CONSIDERATION' OR (status_ = 'APPROVED' AND progress >= $3)) LIMIT 1"#,
            demon.id,
            player.id,
            self.progress
        )
            .fetch_optional(&mut *connection)
            .await?;

        if let Some(row) = existing {
            return Err(DemonlistError::SubmissionExists {
                existing: row.id,
                status: RecordStatus::from_sql(&row.status_),
            })
        }

        Ok(ValidatedSubmission {
            progress: self.progress,
            video,
            status: self.status,
            player,
            demon,
            submitter,
            note: self.note,
        })
    }
}

impl ValidatedSubmission {
    pub async fn create(self, connection: &mut PgConnection) -> Result<FullRecord> {
        let id = sqlx::query(
            "INSERT INTO records (progress, video, status_, player, submitter, demon) VALUES ($1, $2::TEXT, 'SUBMITTED', $3, $4,$5) \
             RETURNING id",
        )
        .bind(self.progress)
        .bind(&self.video)
        .bind(self.player.id)
        .bind(self.submitter.id)
        .bind(self.demon.id)
        .fetch_one(&mut *connection)
        .await?
        .get("id");

        let mut record = FullRecord {
            id,
            progress: self.progress,
            video: self.video,
            status: RecordStatus::Submitted,
            player: self.player,
            demon: self.demon,
            submitter: Some(self.submitter),
            notes: Vec::new(),
        };

        // Dealing with different status and upholding their invariant is complicated, we should not
        // duplicate that code!
        if self.status != RecordStatus::Submitted {
            record.set_status(self.status, &mut *connection).await?;
        }

        if let Some(note) = self.note {
            if !note.trim().is_empty() {
                let note_id = sqlx::query!(
                    "INSERT INTO record_notes (record, content) VALUES ($1, $2) RETURNING id",
                    record.id,
                    note
                )
                .fetch_one(connection)
                .await?
                .id;

                record.notes.push(Note {
                    id: note_id,
                    record: id,
                    content: note,
                    transferred: false,
                    author: None,
                    editors: Vec::new(),
                })
            }
        }

        Ok(record)
    }
}
/*
impl FullRecord {
    pub async fn create_from(
        submitter: Submitter, submission: Submission, connection: &mut PgConnection, ratelimits: Option<PreparedRatelimits<'_>>,
    ) -> Result<FullRecord> {
        // Check ratelimits before any change is made to the database so that the transaction rollback is
        // easier.
        if let Some(ratelimits) = ratelimits {
            ratelimits.check(RatelimitScope::RecordSubmissionGlobal)?;
            ratelimits.check(RatelimitScope::RecordSubmission)?;
        }
    }
}
*/
