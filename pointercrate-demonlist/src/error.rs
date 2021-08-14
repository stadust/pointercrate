use derive_more::Display;
use log::error;
use pointercrate_core::error::{CoreError, PointercrateError};
use serde::Serialize;
use sqlx::{postgres::PgDatabaseError, Error};

pub type Result<T> = std::result::Result<T, DemonlistError>;

#[derive(Serialize, Display, Debug, Eq, PartialEq, Clone)]
pub enum DemonlistError {
    #[display(fmt = "{}", _0)]
    Core(CoreError),

    #[display(fmt = "No submitter with id {} found", id)]
    SubmitterNotFound { id: i32 },
}

impl std::error::Error for DemonlistError {}

impl PointercrateError for DemonlistError {
    fn error_code(&self) -> u16 {
        match self {
            DemonlistError::Core(core) => core.error_code(),
            DemonlistError::SubmitterNotFound { .. } => 40401,
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
