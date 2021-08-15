use derive_more::Display;
use log::error;
use pointercrate_core::{
    error::{CoreError, PointercrateError},
    permission::Permission,
};
use serde::Serialize;
use sqlx::{postgres::PgDatabaseError, Error};

pub type Result<T> = std::result::Result<T, UserError>;

#[derive(Debug, Display, Serialize, Eq, PartialEq, Clone)]
pub enum UserError {
    #[display(fmt = "{}", _0)]
    Core(CoreError),

    #[display(fmt = "Malformed channel URL")]
    MalformedChannelUrl,

    /// `403 FORBIDDEN` error that contains the permissions the client needs to have to perform the
    /// request
    ///
    /// Error Code `40301`
    #[display(
        fmt = "You do not have the pointercrate permissions to perform this request. Required is: {}, which isn't contained in any of \
               your permissions",
        required
    )]
    MissingPermissions {
        /// The permissions required to perform the request
        required: &'static Permission,
    },

    #[display(fmt = "No user with id {} found", user_id)]
    UserNotFound { user_id: i32 },

    #[display(fmt = "No user with name {} found", user_name)]
    UserNotFoundName { user_name: String },

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
            MissingPermissions { .. } => 40301,
            UserNotFound { .. } => 40401,
            UserNotFoundName { .. } => 40401,
            NameTaken => 40902,
            InvalidUsername => 42202,
            InvalidPassword => 42204,
            NotYouTube => 42226,
        }
    }
}

impl From<sqlx::Error> for UserError {
    fn from(error: sqlx::Error) -> Self {
        match error {
            Error::Database(database_error) => {
                let database_error = database_error.downcast::<PgDatabaseError>();

                error!("Database error: {:?}. ", database_error);

                CoreError::DatabaseError
            },
            Error::PoolClosed | Error::PoolTimedOut => CoreError::DatabaseConnectionError,
            Error::ColumnNotFound(column) => {
                error!("Invalid access to column {}, which does not exist", column);

                CoreError::InternalServerError
            },
            Error::RowNotFound => {
                error!("Unhandled 'NotFound', this is a logic or data consistency error");

                CoreError::InternalServerError
            },
            _ => {
                error!("Database error: {:?}", error);

                CoreError::DatabaseError
            },
        }
        .into()
    }
}
