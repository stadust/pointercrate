use super::{Record, RecordStatus};
use crate::{
    config::{EXTENDED_LIST_SIZE, LIST_SIZE},
    error::PointercrateError,
    model::{Demon, Player, Submitter},
    operation::{Delete, Get, Post},
    video, Result,
};
use diesel::{Connection, PgConnection, RunQueryDsl};
use log::{debug, info};
use serde_derive::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Submission {
    pub progress: i16,
    pub player: String,
    pub demon: String,
    #[serde(default)]
    pub video: Option<String>,
    #[serde(rename = "check", default)]
    pub verify_only: bool,
}

impl Post<(Submission, Submitter)> for Option<Record> {
    fn create_from(
        (
            Submission {
                progress,
                player,
                demon,
                video,
                verify_only,
            },
            submitter,
        ): (Submission, Submitter),
        connection: &PgConnection,
    ) -> Result<Self> {
        // Banned submitters cannot submit records
        if submitter.banned {
            return Err(PointercrateError::BannedFromSubmissions)?
        }

        // Check if a video exists and validate it
        let video = match video {
            Some(ref video) => Some(video::validate(video)?),
            None => None,
        };

        connection.transaction(||{
            // Resolve player and demon name against the database
            let player = Player::get(player.as_ref(), connection)?;
            let demon = Demon::get(demon.as_ref(), connection)?;

            // Banned player can't have records on the list
            if player.banned {
                return Err(PointercrateError::PlayerBanned)
            }

            // Cannot submit records for the legacy lsit
            if demon.position > *EXTENDED_LIST_SIZE {
                return Err(PointercrateError::SubmitLegacy)
            }

            // Can only submit 100% records for the extended list
            if demon.position > *LIST_SIZE && progress != 100 {
                return Err(PointercrateError::Non100Extended)
            }

            // Check if the record meets the record requirement for this demon
            if progress > 100 || progress < demon.requirement {
                return Err(PointercrateError::InvalidProgress {
                    requirement: demon.requirement,
                })?
            }

            debug!("Submission is valid, checking for duplicates!");

            // Search for existing records. If no video is provided, we check if a record with the same
            // (demon, player) combination exists. If a video exists, we also check if a record with
            // exactly that video exists. Note that in the second case, two records can be matched,
            // which is why we need the loop here
            let records: Vec<Record> = match video {
                Some(ref video) =>
                    Record::get_existing(player.id, &demon.name, video).get_results(connection)?,
                None => Record::by_player_and_demon(player.id, &demon.name).get_results(connection)?,
            };

            let video_ref = video.as_ref().map(AsRef::as_ref);
            let mut to_delete = Vec::new();

            for record in records {
                // If we have a duplicate it has one of the three record stats. If its rejected, we
                // reject the submission instantly. If it approved, we reject the submission if the
                // approved record has higher progress than the submission. If its submitted, we do the
                // same, but at the same time, we mark the existing submission with lower progress for
                // deleting.
                if record.status() != RecordStatus::Rejected && record.progress() < progress {
                    if record.status() == RecordStatus::Submitted {
                        to_delete.push(record)
                    }
                } else {
                    return Err(PointercrateError::SubmissionExists {
                        status: record.status(),
                        existing: record.id,
                    })
                }
            }

            if verify_only {
                return Ok(None)
            }

            // If none of the duplicates caused us to reject the submission, we now delete submissions marked for deleting
            for record in to_delete {
                debug!(
                    "The submission is duplicated, but new one has higher progress. Deleting old with id {}!",
                    record.id
                );

                record.delete(connection)?;
            }

            debug!("All duplicates either already accepted, or has lower progress, accepting!");

            let id = Record::insert(
                progress,
                video_ref,
                player.id,
                submitter.id,
                &demon.name,
                connection
            )?;

            info!("Submission successful! Created new record with ID {}", id);

            Ok(Some(Record {
                id,
                progress,
                video,
                status: RecordStatus::Submitted,
                player,
                submitter: submitter.id,
                demon: demon.into(),
            }))
        })
    }
}
