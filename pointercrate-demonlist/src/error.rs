use crate::{demon::MinimalDemon, record::RecordStatus};
use derive_more::Display;

use pointercrate_core::error::{CoreError, PointercrateError};
use serde::Serialize;

pub type Result<T> = std::result::Result<T, DemonlistError>;

#[derive(Serialize, Display, Debug, Eq, PartialEq, Clone)]
#[serde(untagged)]
pub enum DemonlistError {
    #[display(fmt = "{}", _0)]
    Core(CoreError),

    #[display(fmt = "Malformed video URL")]
    MalformedVideoUrl,

    /// `403 FORBIDDEN` error returned if someone with an IP-address that's banned from submitting
    /// records tries to submit a record
    ///
    /// Error Code `40304`
    #[display(fmt = "You are banned from submitting records to the demonlist!")]
    BannedFromSubmissions,

    #[display(fmt = "You claim on this player is unverified")]
    ClaimUnverified,

    #[display(fmt = "IP Geolocation attempt through VPS detected")]
    VpsDetected,

    /// `403 FORBIDDEN` variant returned when someone tries to submit a records for a player who
    /// does not permit others to submit their records.
    ///
    /// This means only the pointercrate user who has a claim on the given player, while logged in,
    /// can submit records for this player.
    ///
    /// Error Code `40308`
    #[display(fmt = "This player has requested that only they themselves can submit their records")]
    NoThirdPartySubmissions,

    #[display(fmt = "No submitter with id {} found", id)]
    SubmitterNotFound { id: i32 },

    #[display(fmt = "No note with id {} found on record with id {}", note_id, record_id)]
    NoteNotFound { note_id: i32, record_id: i32 },

    #[display(fmt = "Player with id {} is no creator of demon with id {}", player_id, demon_id)]
    CreatorNotFound { demon_id: i32, player_id: i32 },

    #[display(fmt = "No nationality with iso code {} found", iso_code)]
    NationalityNotFound { iso_code: String },

    #[display(fmt = "No subdivision with code {} found in nation {}", subdivision_code, nation_code)]
    SubdivisionNotFound { subdivision_code: String, nation_code: String },

    #[display(fmt = "No player with id {} found", player_id)]
    PlayerNotFound { player_id: i32 },

    #[display(fmt = "No player with name {} found", player_name)]
    PlayerNotFoundName { player_name: String },

    #[display(fmt = "No demon with id {} found", demon_id)]
    DemonNotFound { demon_id: i32 },

    #[display(fmt = "No demon with name {} found", demon_name)]
    DemonNotFoundName { demon_name: String },

    #[display(fmt = "No demon at position {} found", demon_position)]
    DemonNotFoundPosition { demon_position: i16 },

    #[display(fmt = "No record with id {} found", record_id)]
    RecordNotFound { record_id: i32 },

    #[display(fmt = "No claim by user {} on player {} found", member_id, player_id)]
    ClaimNotFound { member_id: i32, player_id: i32 },

    #[display(fmt = "This player is already registered as a creator on this demon")]
    CreatorExists,

    /// `409 CONFLICT` variant
    ///
    /// Error Code `40906`
    #[display(fmt = "This video is already used by record #{}", id)]
    DuplicateVideo { id: i32 },

    /// `409 CONFLICT` variant
    ///
    /// Error Code `40907`
    #[display(fmt = "Attempt to set subdivision without nation")]
    NoNationSet,

    #[display(
        fmt = "The players '{}' and '{}' have verified claims by different pointercrate users",
        player1,
        player2
    )]
    ConflictingClaims { player1: String, player2: String },

    /// `422 UNPROCESSABLE ENTITY` variant returned if attempted to create a demon with a record
    /// requirements outside of [0, 100]
    ///
    /// Error Code `42212`
    #[display(fmt = "Record requirement needs to be greater than -1 and smaller than 101")]
    InvalidRequirement,

    /// `422 UNPROCESSABLE ENTITY` variant returned if attempted to create a demon with a position,
    /// that would leave "holes" in the list, or is smaller than 1
    ///
    /// Error Code `42213`
    #[display(
        fmt = "Demon position needs to be greater than or equal to 1 and smaller than or equal to {}",
        maximal
    )]
    InvalidPosition {
        /// The maximal position a new demon can be added at
        maximal: i16,
    },

    /// `422 UNPROCESSABLE ENTITY` variant
    ///
    /// Error Code `42215`
    #[display(fmt = "Record progress must lie between {} and 100%!", requirement)]
    InvalidProgress {
        /// The [`Demon`]'s record requirement
        requirement: i16,
    },
    /// `422 UNPROCESSABLE ENTITY` variant
    ///
    /// Error Code `42217`
    #[display(fmt = "This record is already {} (existing record: {})", status, existing)]
    SubmissionExists {
        /// The [`RecordStatus`] of the existing [`Record`]
        status: RecordStatus,

        /// The ID of the existing [`Record`]
        existing: i32,
    },

    /// `422 UNPROCESSABLE ENTITY` variant
    ///
    /// Error Code `42218`
    #[display(fmt = "The given player is banned and thus cannot have non-rejected records on the list!")]
    PlayerBanned,

    /// `422 UNPROCESSABLE ENTITY` variant
    ///
    /// Error Code 42219
    #[display(fmt = "You cannot submit records for legacy demons")]
    SubmitLegacy,

    /// `422 UNPROCESSABLE ENTITY` variant
    ///
    /// Error Code 42220
    #[display(fmt = "Only 100% records can be submitted for the extended section of the list")]
    Non100Extended,

    /// `422 UNPROCESSABLE ENTITY` variant
    ///
    /// Error Code `42224`
    #[display(fmt = "The given video host is not supported. Supported are 'youtube', 'vimeo', 'everyplay', 'twitch' and 'bilibili'")]
    UnsupportedVideoHost,

    /// `422 UNPROCESSABLE ENTITY` variant
    ///
    /// Error Code `42228`
    #[display(fmt = "There are multiple demons with the given name")]
    DemonNameNotUnique { demons: Vec<MinimalDemon> },

    /// `422 UNPROCESSABLE ENTITY` variant
    ///
    /// Error Code `42230`
    #[display(fmt = "Notes mustn't be empty!")]
    NoteEmpty,

    /// `422 UNPROCESSABLE ENTITY` variant
    ///
    /// Error Code `42231`
    #[display(fmt = "This player already have a verified claim associated with them")]
    AlreadyClaimed,

    /// `422 UNPROCESSABLE ENTITY` variant
    ///
    /// Error Code `42232`
    #[display(fmt = "Raw footage much be provided to submit this record")]
    RawRequired, //hehe

    /// `422 UNPROCESSABLE ENTITY` variant
    ///
    /// Error Code `42233`
    #[display(fmt = "Raw footage needs to be a valid URL")]
    MalformedRawUrl,

    /// `422 UNPROCESSABLE ENTITY` variant
    ///
    /// Error Code `42235`
    #[display(fmt = "Level ID needs to be positive")]
    InvalidLevelId,
}

impl std::error::Error for DemonlistError {}

impl PointercrateError for DemonlistError {
    fn error_code(&self) -> u16 {
        use DemonlistError::*;

        match self {
            Core(core) => core.error_code(),
            SubmitterNotFound { .. } => 40401,
            NoteNotFound { .. } => 40401,
            CreatorNotFound { .. } => 40401,
            CreatorExists => 40905,
            InvalidRequirement => 42212,
            InvalidPosition { .. } => 42213,
            NoteEmpty => 42230,
            MalformedVideoUrl => 40001,
            BannedFromSubmissions => 40304,
            ClaimUnverified => 40306,
            VpsDetected => 40307,
            NoThirdPartySubmissions => 40308,
            NationalityNotFound { .. } => 40401,
            SubdivisionNotFound { .. } => 40401,
            PlayerNotFound { .. } => 40401,
            PlayerNotFoundName { .. } => 40401,
            DemonNotFound { .. } => 40401,
            DemonNotFoundName { .. } => 40401,
            DemonNotFoundPosition { .. } => 40401,
            RecordNotFound { .. } => 40401,
            ClaimNotFound { .. } => 40401,
            DuplicateVideo { .. } => 40906,
            NoNationSet => 40907,
            ConflictingClaims { .. } => 40908,
            InvalidProgress { .. } => 42215,
            SubmissionExists { .. } => 42217,
            PlayerBanned => 42218,
            SubmitLegacy => 42219,
            Non100Extended => 42220,
            UnsupportedVideoHost => 42224,
            DemonNameNotUnique { .. } => 42228,
            AlreadyClaimed => 42231,
            RawRequired => 42232,
            MalformedRawUrl => 42233,
            InvalidLevelId => 42234,
        }
    }
}

impl From<CoreError> for DemonlistError {
    fn from(error: CoreError) -> Self {
        DemonlistError::Core(error)
    }
}

impl From<sqlx::Error> for DemonlistError {
    fn from(error: sqlx::Error) -> Self {
        DemonlistError::Core(error.into())
    }
}
