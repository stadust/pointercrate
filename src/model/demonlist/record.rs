pub use self::{
    get::{approved_records_by, approved_records_on, submitted_by},
    paginate::RecordPagination,
    patch::PatchRecord,
    post::Submission,
};
use crate::{
    cistring::CiString,
    error::PointercrateError,
    model::demonlist::{demon::MinimalDemon, player::DatabasePlayer, submitter::Submitter},
    ratelimit::RatelimitScope::RecordSubmission,
    state::PointercrateState,
    Result,
};
use derive_more::Display;
use log::{debug, error, info, warn};
use reqwest::Client;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::json;
use sqlx::PgConnection;
use std::{
    fmt::{Display, Formatter},
    str::FromStr,
};

mod delete;
mod get;
mod paginate;
mod patch;
mod post;

#[derive(Debug, Eq, PartialEq, Clone, Copy, Hash)]
pub enum RecordStatus {
    Submitted,
    Approved,
    Rejected,
}

impl Default for RecordStatus {
    fn default() -> Self {
        RecordStatus::Submitted
    }
}

impl Display for RecordStatus {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            RecordStatus::Submitted => write!(f, "submitted"),
            RecordStatus::Approved => write!(f, "approved"),
            RecordStatus::Rejected => write!(f, "rejected"),
        }
    }
}

impl FromStr for RecordStatus {
    type Err = PointercrateError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match &s.to_lowercase()[..] {
            "submitted" => Ok(RecordStatus::Submitted),
            "approved" => Ok(RecordStatus::Approved),
            "rejected" => Ok(RecordStatus::Rejected),
            _ =>
                Err(PointercrateError::InvalidInternalStateError {
                    cause: "Encountered a record state other than 'approved', 'submitted' or 'rejected'",
                }),
        }
    }
}

impl Serialize for RecordStatus {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for RecordStatus {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let string = String::deserialize(deserializer)?.to_lowercase();

        match &string[..] {
            "approved" => Ok(RecordStatus::Approved),
            "submitted" => Ok(RecordStatus::Submitted),
            "rejected" => Ok(RecordStatus::Rejected),
            _ =>
                Err(serde::de::Error::invalid_value(
                    serde::de::Unexpected::Str(&string),
                    &"'approved', 'submitted' or 'rejected'",
                )),
        }
    }
}

#[derive(Debug, Serialize, Hash, Display)]
#[display(fmt = "{} {}% on {} (ID: {})", player, progress, demon, id)]
pub struct FullRecord {
    pub id: i32,
    pub progress: i16,
    pub video: Option<String>,
    pub status: RecordStatus,
    pub player: DatabasePlayer,
    pub demon: MinimalDemon,
    pub submitter: Option<Submitter>,
    pub notes: Option<String>,
}

#[derive(Debug, Hash, Serialize, Display)]
#[display(fmt = "{} {}% on {} (ID: {})", player, progress, demon, id)]
pub struct MinimalRecordPD {
    pub id: i32,
    pub progress: i16,
    pub video: Option<String>,
    pub status: RecordStatus,
    pub demon: MinimalDemon,
    pub player: DatabasePlayer,
}

#[derive(Debug, Hash, Serialize, Display)]
#[display(fmt = "{}% on {} (ID: {})", progress, demon, id)]
pub struct MinimalRecordD {
    pub id: i32,
    pub progress: i16,
    pub video: Option<String>,
    pub status: RecordStatus,
    pub demon: MinimalDemon,
}

#[derive(Debug, Hash, Serialize, Display)]
#[display(fmt = "{} - {}% (ID: {})", player, progress, id)]
pub struct MinimalRecordP {
    pub id: i32,
    pub progress: i16,
    pub video: Option<String>,
    pub status: RecordStatus,
    pub player: DatabasePlayer,
}

impl FullRecord {
    /// Gets the maximal and minimal submitter id currently in use
    ///
    /// The returned tuple is of the form (max, min)
    pub async fn extremal_record_ids(connection: &mut PgConnection) -> Result<(i32, i32)> {
        let row = sqlx::query!("SELECT MAX(id) AS max_id, MIN(id) AS min_id FROM records")
            .fetch_one(connection)
            .await?; // FIXME: crashes on empty table
        Ok((row.max_id, row.min_id))
    }

    pub async fn validate(self, state: PointercrateState) {
        let mut connection = match state.connection().await {
            Ok(connection) => connection,
            Err(err) => return error!("INTERNAL SERVER ERROR: failed to acquire database connection: {:?}", err),
        };

        let video = match self.video {
            Some(ref video) => video,
            None => return,
        };

        debug!("Verifying that submission {} with video {} actually is valid", self, video);

        match state.http_client.head(video).send().await {
            Ok(response) => {
                let status = response.status().as_u16();

                info!("Record valid!");

                if status == 401 || status == 403 || status == 405 {
                    // Some websites (billibilli) respond unfavorably to HEAD requests. Retry with
                    // GET
                    match state.http_client.get(video).send().await {
                        Ok(response) => {
                            let status = response.status().as_u16();

                            if status >= 200 && status < 400 {
                                debug!("HEAD request yielded some sort of successful response, executing webhook");

                                self.execute_webhook(&state).await;
                            }
                        },
                        Err(err) => {
                            error!(
                                "INTERNAL SERVER ERROR: HEAD request to verify video failed: {:?}. Deleting submission",
                                err
                            );

                            match self.delete(&mut connection).await {
                                Ok(_) => (),
                                Err(error) => error!("INTERNAL SERVER ERROR: Failure to delete record - {:?}!", error),
                            }
                        },
                    }
                } else if status >= 200 && status < 400 {
                    debug!("HEAD request yielded some sort of successful response, executing webhook");

                    self.execute_webhook(&state).await;
                } else {
                    warn!("Server response to 'HEAD {}' was {:?}, deleting submission!", video, response);

                    match self.delete(&mut connection).await {
                        Ok(_) => (),
                        Err(error) => error!("INTERNAL SERVER ERROR: Failure to delete record - {:?}!", error),
                    }
                }
            },
            Err(error) => {
                error!(
                    "INTERNAL SERVER ERROR: HEAD request to verify video failed: {:?}. Deleting submission",
                    error
                );

                match self.delete(&mut connection).await {
                    Ok(_) => (),
                    Err(error) => error!("INTERNAL SERVER ERROR: Failure to delete record - {:?}!", error),
                }
            },
        }
    }

    async fn execute_webhook(&self, state: &PointercrateState) {
        if let Some(ref webhook_url) = state.webhook_url {
            match state
                .http_client
                .post(&**webhook_url)
                .header("Content-Type", "application/json")
                .body(self.webhook_embed().to_string())
                .send()
                .await
            {
                Err(error) => error!("INTERNAL SERVER ERROR: Failure to execute discord webhook: {:?}", error),
                Ok(_) => debug!("Successfully executed discord webhook"),
            }
        } else {
            warn!("Trying to execute webhook, though no link was configured!");
        }
    }

    fn webhook_embed(&self) -> serde_json::Value {
        let progress = f32::from(self.progress) / 100f32;

        let mut payload = json!({
            "content": format!("**New record submitted! ID: {}**", self.id),
            "embeds": [
                {
                    "type": "rich",
                    "title": format!("{}% on {}", self.progress, self.demon.name),
                    "description": format!("{} just got {}% on {}! Go add his record!", self.player.name, self.progress, self.demon.name),
                    "footer": {
                        "text": format!("This record has been submitted by submitter #{}", self.submitter.map(|s|s.id).unwrap_or(1))
                    },
                    "color": ((158 as f32 * progress) as i32) << 16 + ((224 as f32 * progress) as i32) << 8,
                    "author": {
                        "name": format!("{} (ID: {})", self.player.name, self.player.id),
                        "url": self.video
                    },
                    "thumbnail": {
                        "url": "https://cdn.discordapp.com/avatars/277391246035648512/b03c85d94dc02084c413a7fdbe2cea79.webp?size=1024"
                    },
                }
            ]
        });

        if let Some(ref video) = self.video {
            payload["embeds"][0]["fields"] = json! {
                [{
                    "name": "Video Proof:",
                    "value": video
                }]
            };
        }

        payload
    }
}
