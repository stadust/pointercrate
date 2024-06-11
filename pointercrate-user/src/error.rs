use derive_more::Display;

use pointercrate_core::{
    error::{CoreError, PointercrateError},
    permission::Permission,
};
use serde::Serialize;
use std::collections::HashSet;

pub type Result<T> = std::result::Result<T, UserError>;

#[derive(Debug, Display, Serialize, Eq, PartialEq, Clone)]
pub enum UserError {
    #[display(fmt = "{}", _0)]
    Core(CoreError),

    #[display(fmt = "Malformed channel URL")]
    MalformedChannelUrl,

    /// `403 FORBIDDEN` error returned when a user attempts to delete his own account via the admin
    /// panel
    ///
    /// Error Code `40302`
    #[display(fmt = "You cannot delete your own account via this endpoint. Use DELETE /api/v1/auth/me/")]
    DeleteSelf,

    /// `403 FORBIDDEN` error returned when a user attempts to patch his own account via the admin
    /// panel
    ///
    /// Error Code `40303`
    #[display(fmt = "You cannot modify your own account via this endpoint. Use PATCH /api/v1/auth/me/")]
    PatchSelf,

    #[display(fmt = "You cannot assign the following permissions: {:?}", non_assignable)]
    PermissionNotAssignable { non_assignable: HashSet<Permission> },

    #[display(fmt = "No user with id {} found", user_id)]
    UserNotFound { user_id: i32 },

    #[display(fmt = "No user with name {} found", user_name)]
    UserNotFoundName { user_name: String },

    #[display(fmt = "No user with google account {} found", google_account_id)]
    UserNotFoundGoogleAccount { google_account_id: String },

    /// `409 CONFLICT` error returned if a user tries to register with a name that's already taken
    ///
    /// Error Code `40902`
    #[display(fmt = "The chosen username is already taken")]
    NameTaken,

    /// `422 UNPROCESSABLE ENTITIY` variant returned if the username provided during registration
    /// is either shorter than 3 letters of contains trailing or leading whitespaces
    ///
    /// Error Code: `42202`
    #[display(fmt = "Invalid display- or username! The name must be at least 3 characters long and not start/end with a space")]
    InvalidUsername,

    /// `422 UNPROCESSABLE ENTITY` variant returned if the password provided during registration
    /// (or account update) is shorter than 10 characters
    ///
    /// Error Code `42204`
    #[display(fmt = "Invalid password! The password must be at least 10 characters long")]
    InvalidPassword,

    /// `422 UNPROCESSABLE ENTITY` variant
    ///
    /// Error Code `42226`
    #[display(fmt = "The given URL is no YouTube URL")]
    NotYouTube,
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
            UserNotFoundGoogleAccount { .. } => 40401,
            NameTaken => 40902,
            InvalidUsername => 42202,
            InvalidPassword => 42204,
            NotYouTube => 42226,
        }
    }
}

impl From<sqlx::Error> for UserError {
    fn from(error: sqlx::Error) -> Self {
        UserError::Core(error.into())
    }
}
