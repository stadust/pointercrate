use crate::{
    cistring::CiString,
    config,
    error::PointercrateError,
    model::demonlist::{
        demon::MinimalDemon,
        player::DatabasePlayer,
        record::{FullRecord, RecordStatus},
        submitter::Submitter,
    },
    ratelimit::{PreparedRatelimits, RatelimitScope},
    Result,
};
use derive_more::Display;
use log::{debug, info};
use serde::Deserialize;
use sqlx::{PgConnection, Row};
use std::str::FromStr;

#[derive(Deserialize, Debug, Display)]
#[display(fmt = "{}% on {} by {} [status: {}]", progress, demon, player, status)]
pub struct Submission {
    pub progress: i16,
    pub player: CiString,
    pub demon: CiString,
    #[serde(default)]
    pub video: Option<String>,
    #[serde(default)]
    pub status: RecordStatus,
}

impl FullRecord {
    pub async fn create_from(
        submitter: Submitter, submission: Submission, connection: &mut PgConnection, ratelimits: Option<PreparedRatelimits<'_>>,
    ) -> Result<FullRecord> {
        info!("Processing record addition '{}' by {}", submission, submitter);

        // Banned submitters cannot submit records
        if submitter.banned {
            return Err(PointercrateError::BannedFromSubmissions)
        }

        let video = match submission.video {
            Some(ref video) => Some(crate::video::validate(video)?),
            None => None,
        };

        // Resolve player and demon name against the database
        let player = DatabasePlayer::by_name_or_create(submission.player.as_ref(), connection).await?;
        // TODO: handle the ambiguous case
        let demon = MinimalDemon::by_name(submission.demon.as_ref(), connection).await?;

        // Banned player can't have records on the list
        if player.banned {
            return Err(PointercrateError::PlayerBanned)
        }

        // Cannot submit records for the legacy list (it is possible to directly add them for list mods)
        if demon.position > config::extended_list_size() && submission.status == RecordStatus::Submitted {
            return Err(PointercrateError::SubmitLegacy)
        }

        // Can only submit 100% records for the extended list (it is possible to directly add them for list
        // mods)
        if demon.position > config::list_size() && submission.progress != 100 && submission.status == RecordStatus::Submitted {
            return Err(PointercrateError::Non100Extended)
        }

        let requirement = demon.requirement(connection).await?;

        // Check if the record meets the record requirement for this demon
        if submission.progress > 100 || submission.progress < requirement {
            return Err(PointercrateError::InvalidProgress { requirement })
        }

        debug!("Submission is valid, checking for duplicates!");

        // Search for existing records. If no video is provided, we check if a record with the same
        // (demon, player) combination exists. If a video exists, we also check if a record with
        // exactly that video exists. Note that in the second case, two records can be matched,
        // which is why we need the loop here

        if let Some(ref video) = submission.video {
            if let Some(row) = sqlx::query!("SELECT id, status_::text FROM records WHERE video = $1", video.to_string())
                .fetch_optional(connection) // FIXME(sqlx)
                .await?
            {
                return Err(PointercrateError::SubmissionExists {
                    existing: row.id,
                    status: RecordStatus::from_str(&row.status_)?,
                })
            }
        }

        if let Some(row) = sqlx::query!(
            "SELECT id, status_::text FROM records WHERE demon = $1 AND player = $2 AND (status_ = 'REJECTED' OR status_='SUBMITTED' AND \
             progress >= $3 OR status_ = CAST($4::TEXT AS record_status) AND progress >= $3) LIMIT 1",
            demon.id,
            player.id,
            submission.progress,
            submission.status.to_string()
        )
        .fetch_optional(connection)
        .await?
        {
            return Err(PointercrateError::SubmissionExists {
                existing: row.id,
                status: RecordStatus::from_str(&row.status_)?,
            })
        }

        // Check ratelimits before any change is made to the database so that the transaction rollback is
        // easier.
        if let Some(ratelimits) = ratelimits {
            ratelimits.check(RatelimitScope::RecordSubmissionGlobal)?;
            ratelimits.check(RatelimitScope::RecordSubmission)?;
        }

        sqlx::query!(
            "DELETE FROM records WHERE status_ = CAST($1::TEXT as record_status) AND progress < $2",
            submission.status.to_string(),
            submission.progress
        )
        .execute(connection)
        .await?; // TOdO: handle notes (once the new system is in-place)

        let id = sqlx::query(
            "INSERT INTO records (progress, video, status_, player, submitter, demon) VALUES ($1, $2::TEXT, CAST($3::TEXT AS \
             record_status), $4, $5,$6) RETURNING id",
        )
        .bind(submission.progress)
        .bind(&video)
        .bind(submission.status.to_string())
        .bind(player.id)
        .bind(submitter.id)
        .bind(demon.id)
        .fetch_one(connection)
        .await?
        .get("id");

        Ok(FullRecord {
            id,
            progress: submission.progress,
            video,
            status: submission.status,
            player,
            demon,
            submitter: Some(submitter),
            notes: None,
        })
    }
}
