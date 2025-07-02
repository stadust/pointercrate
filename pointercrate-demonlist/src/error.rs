use std::fmt::Display;

use crate::{demon::MinimalDemon, record::RecordStatus};

use pointercrate_core::{
    error::{CoreError, PointercrateError},
    localization::tr,
    trp,
};
use serde::Serialize;

pub type Result<T> = std::result::Result<T, DemonlistError>;

#[derive(Serialize, Debug, Eq, PartialEq, Clone)]
#[serde(untagged)]
pub enum DemonlistError {
    Core(CoreError),

    MalformedVideoUrl,

    /// `403 FORBIDDEN` error returned if someone with an IP-address that's banned from submitting
    /// records tries to submit a record
    ///
    /// Error Code `40304`
    BannedFromSubmissions,

    ClaimUnverified,

    VpsDetected,

    /// `403 FORBIDDEN` variant returned when someone tries to submit a records for a player who
    /// does not permit others to submit their records.
    ///
    /// This means only the pointercrate user who has a claim on the given player, while logged in,
    /// can submit records for this player.
    ///
    /// Error Code `40308`
    NoThirdPartySubmissions,

    SubmitterNotFound {
        id: i32,
    },

    NoteNotFound {
        note_id: i32,
        record_id: i32,
    },

    CreatorNotFound {
        demon_id: i32,
        player_id: i32,
    },

    NationalityNotFound {
        iso_code: String,
    },

    SubdivisionNotFound {
        subdivision_code: String,
        nation_code: String,
    },

    PlayerNotFound {
        player_id: i32,
    },

    PlayerNotFoundName {
        player_name: String,
    },

    DemonNotFound {
        demon_id: i32,
    },

    DemonNotFoundName {
        demon_name: String,
    },

    DemonNotFoundPosition {
        demon_position: i16,
    },

    RecordNotFound {
        record_id: i32,
    },

    ClaimNotFound {
        member_id: i32,
        player_id: i32,
    },

    CreatorExists,

    /// `409 CONFLICT` variant
    ///
    /// Error Code `40906`
    DuplicateVideo {
        id: i32,
    },

    /// `409 CONFLICT` variant
    ///
    /// Error Code `40907`
    NoNationSet,

    ConflictingClaims {
        player1: String,
        player2: String,
    },

    /// `422 UNPROCESSABLE ENTITY` variant returned if attempted to create a demon with a record
    /// requirements outside of [0, 100]
    ///
    /// Error Code `42212`
    InvalidRequirement,

    /// `422 UNPROCESSABLE ENTITY` variant returned if attempted to create a demon with a position,
    /// that would leave "holes" in the list, or is smaller than 1
    ///
    /// Error Code `42213`
    InvalidPosition {
        /// The maximal position a new demon can be added at
        maximal: i16,
    },

    /// `422 UNPROCESSABLE ENTITY` variant
    ///
    /// Error Code `42215`
    InvalidProgress {
        /// The [`Demon`]'s record requirement
        requirement: i16,
    },
    /// `422 UNPROCESSABLE ENTITY` variant
    ///
    /// Error Code `42217`
    SubmissionExists {
        /// The [`RecordStatus`] of the existing [`Record`]
        status: RecordStatus,

        /// The ID of the existing [`Record`]
        existing: i32,
    },

    /// `422 UNPROCESSABLE ENTITY` variant
    ///
    /// Error Code `42218`
    PlayerBanned,

    /// `422 UNPROCESSABLE ENTITY` variant
    ///
    /// Error Code 42219
    SubmitLegacy,

    /// `422 UNPROCESSABLE ENTITY` variant
    ///
    /// Error Code 42220
    Non100Extended,

    /// `422 UNPROCESSABLE ENTITY` variant
    ///
    /// Error Code `42224`
    UnsupportedVideoHost,

    /// `422 UNPROCESSABLE ENTITY` variant
    ///
    /// Error Code `42228`
    DemonNameNotUnique {
        demons: Vec<MinimalDemon>,
    },

    /// `422 UNPROCESSABLE ENTITY` variant
    ///
    /// Error Code `42230`
    NoteEmpty,

    /// `422 UNPROCESSABLE ENTITY` variant
    ///
    /// Error Code `42231`
    AlreadyClaimed,

    /// `422 UNPROCESSABLE ENTITY` variant
    ///
    /// Error Code `42232`
    RawRequired, //hehe

    /// `422 UNPROCESSABLE ENTITY` variant
    ///
    /// Error Code `42233`
    MalformedRawUrl,

    /// `422 UNPROCESSABLE ENTITY` variant
    ///
    /// Error Code `42235`
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
            InvalidLevelId => 42235,
        }
    }
}

impl Display for DemonlistError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                DemonlistError::Core(core) => {
                    return core.fmt(f);
                },
                DemonlistError::MalformedVideoUrl => tr("error-demonlist-malformedvideourl"),
                DemonlistError::BannedFromSubmissions => tr("error-demonlist-bannedfromsubmissions"),
                DemonlistError::ClaimUnverified => tr("error-demonlist-claimunverified"),
                DemonlistError::VpsDetected => tr("error-demonlist-vpsdetected"),
                DemonlistError::NoThirdPartySubmissions => tr("nothirdpartysubmissions-error-malformedvideourl"),
                DemonlistError::SubmitterNotFound { id } => trp!("error-demonlist-submitternotfound", ("id", id)),
                DemonlistError::NoteNotFound { note_id, record_id } => {
                    trp!("error-demonlist-notenotfound", ("note-id", note_id), ("record-id", record_id))
                },
                DemonlistError::CreatorNotFound { demon_id, player_id } => {
                    trp!("error-demonlist-creatornotfound", ("player-id", player_id), ("demon-id", demon_id))
                },
                DemonlistError::NationalityNotFound { iso_code } => trp!("error-demonlist-nationalitynotfound", ("iso-code", iso_code)),
                DemonlistError::SubdivisionNotFound {
                    subdivision_code,
                    nation_code,
                } => trp!(
                    "error-demonlist-subdivisionnotfound",
                    ("subdivision-code", subdivision_code),
                    ("nation-code", nation_code)
                ),
                DemonlistError::PlayerNotFound { player_id } => trp!("error-demonlist-playernotfound", ("player-id", player_id)),
                DemonlistError::PlayerNotFoundName { player_name } =>
                    trp!("error-demonlist-playernotfoundname", ("player-name", player_name)),
                DemonlistError::DemonNotFound { demon_id } => trp!("error-demonlist-demonnotfound", ("demon-id", demon_id)),
                DemonlistError::DemonNotFoundName { demon_name } => trp!("error-demonlist-demonnotfoundname", ("demon-name", demon_name)),
                DemonlistError::DemonNotFoundPosition { demon_position } =>
                    trp!("error-demonlist-demonnotfoundposition", ("demon-position", demon_position)),
                DemonlistError::RecordNotFound { record_id } => trp!("error-demonlist-recordnotfound", ("record-id", record_id)),
                DemonlistError::ClaimNotFound { member_id, player_id } =>
                    trp!("error-demonlist-claimnotfound", ("member-id", member_id), ("player-id", player_id)),
                DemonlistError::CreatorExists => tr("error-demonlist-creatorexists"),
                DemonlistError::DuplicateVideo { id } => trp!("error-demonlist-duplicatevideo", ("record-id", id)),
                DemonlistError::NoNationSet => tr("error-demonlist-nonationset"),
                DemonlistError::ConflictingClaims { player1, player2 } =>
                    trp!("error-demonlist-conflictingclaims", ("player-1", player1), ("player-2", player2)),
                DemonlistError::InvalidRequirement => tr("error-demonlist-invalidrequirement"),
                DemonlistError::InvalidPosition { maximal } => trp!("error-demonlist-invalidposition", ("maximal", maximal)),
                DemonlistError::InvalidProgress { requirement } => trp!("error-demonlist-invalidprogress", ("requirement", requirement)),
                DemonlistError::SubmissionExists { status, existing } => trp!(
                    "error-demonlist-submissionexists",
                    ("record-status", format!("{}", status)),
                    ("record-id", existing)
                ),
                DemonlistError::PlayerBanned => tr("error-demonlist-playerbanned"),
                DemonlistError::SubmitLegacy => tr("error-demonlist-submitlegacy"),
                DemonlistError::Non100Extended => tr("error-demonlist-non100extended"),
                DemonlistError::UnsupportedVideoHost => tr("error-demonlist-unsupportedvideohost"),
                DemonlistError::DemonNameNotUnique { .. } => tr("error-demonlist-demonnamenotunique"),
                DemonlistError::NoteEmpty => tr("error-demonlist-noteempty"),
                DemonlistError::AlreadyClaimed => tr("error-demonlist-alreadyclaimed"),
                DemonlistError::RawRequired => tr("error-demonlist-rawrequired"),
                DemonlistError::MalformedRawUrl => tr("error-demonlist-malformedrawurl"),
                DemonlistError::InvalidLevelId => tr("error-demonlist-invalidlevelid"),
            }
        )
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
