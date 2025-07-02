use pointercrate_core::{
    error::{CoreError, PointercrateError},
    localization::tr,
    permission::Permission,
    trp,
};
use serde::Serialize;
use std::{collections::HashSet, fmt::Display};

pub type Result<T> = std::result::Result<T, UserError>;

#[derive(Debug, Serialize, Eq, PartialEq, Clone)]
pub enum UserError {
    Core(CoreError),

    MalformedChannelUrl,

    /// `403 FORBIDDEN` error returned when a user attempts to delete his own account via the admin
    /// panel
    ///
    /// Error Code `40302`
    DeleteSelf,

    /// `403 FORBIDDEN` error returned when a user attempts to patch his own account via the admin
    /// panel
    ///
    /// Error Code `40303`
    PatchSelf,

    PermissionNotAssignable {
        non_assignable: HashSet<Permission>,
    },

    UserNotFound {
        user_id: i32,
    },

    UserNotFoundName {
        user_name: String,
    },

    /// `409 CONFLICT` error returned if a user tries to register with a name that's already taken
    ///
    /// Error Code `40902`
    NameTaken,

    /// `422 UNPROCESSABLE ENTITIY` variant returned if the username provided during registration
    /// is either shorter than 3 letters of contains trailing or leading whitespaces
    ///
    /// Error Code: `42202`
    InvalidUsername,

    /// `422 UNPROCESSABLE ENTITY` variant returned if the password provided during registration
    /// (or account update) is shorter than 10 characters
    ///
    /// Error Code `42204`
    InvalidPassword,

    /// `422 UNPROCESSABLE ENTITY` variant
    ///
    /// Error Code `42226`
    NotYouTube,

    /// `422 UNPROCESSABL ENTITY` variant indicating that the attempted operation can only be
    /// performed on a legacy, password-based account (for example, trying to change password
    /// for a non-legacy account).
    ///
    /// Error Code `42234`
    NonLegacyAccount,
}

impl std::error::Error for UserError {}

impl From<CoreError> for UserError {
    fn from(err: CoreError) -> Self {
        UserError::Core(err)
    }
}

impl PointercrateError for UserError {
    fn error_code(&self) -> u16 {
        use UserError::*;

        match self {
            Core(core) => core.error_code(),

            MalformedChannelUrl => 40001,
            DeleteSelf => 40302,
            PatchSelf => 40303,
            PermissionNotAssignable { .. } => 40305,
            UserNotFound { .. } => 40401,
            UserNotFoundName { .. } => 40401,
            NameTaken => 40902,
            InvalidUsername => 42202,
            InvalidPassword => 42204,
            NotYouTube => 42226,
            NonLegacyAccount => 42234,
        }
    }
}

impl Display for UserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                UserError::Core(core) => {
                    return core.fmt(f);
                },
                UserError::MalformedChannelUrl => tr("error-user-malformedchannelurl"),
                UserError::DeleteSelf => tr("error-user-deleteself"),
                UserError::PatchSelf => tr("error-user-patchself"),
                UserError::PermissionNotAssignable { non_assignable } => trp!(
                    "error-user-permissionnotassignable",
                    (
                        "non-assignable",
                        non_assignable
                            .iter()
                            .map(|permission| tr(permission.text_id()))
                            .collect::<Vec<String>>()
                            .join(", ")
                    )
                ),
                UserError::UserNotFound { user_id } => trp!("error-user-usernotfound", ("user-id", user_id)),
                UserError::UserNotFoundName { user_name } => trp!("error-user-usernotfoundname", ("user-name", user_name)),
                UserError::NameTaken => tr("error-user-nametaken"),
                UserError::InvalidUsername => tr("error-user-invalidusername"),
                UserError::InvalidPassword => tr("error-user-invalidpassword"),
                UserError::NotYouTube => tr("error-user-notyoutube"),
                UserError::NonLegacyAccount => tr("error-user-nonlegacyaccount"),
            }
        )
    }
}

impl From<sqlx::Error> for UserError {
    fn from(error: sqlx::Error) -> Self {
        UserError::Core(error.into())
    }
}
