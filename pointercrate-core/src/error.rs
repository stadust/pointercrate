use crate::permission::Permission;
use derive_more::Display;
use log::error;
use serde::Serialize;
use std::{error::Error, time::Duration};

pub type Result<T> = std::result::Result<T, CoreError>;

pub trait PointercrateError: Error + Serialize + From<CoreError> {
    fn error_code(&self) -> u16;
    fn status_code(&self) -> u16 {
        self.error_code() / 100
    }
}

#[derive(Serialize, Display, Debug, Eq, PartialEq, Clone)]
#[serde(untagged)]
pub enum CoreError {
    /// Generic `400 BAD REQUEST` error
    ///
    /// Error Code `40000`
    #[display(fmt = "The browser (or proxy) sent a request that this server could not understand.")]
    BadRequest,

    /// `400 BAD REQUEST' error returned when a header value was malformed
    ///
    /// Error Code `40002`
    #[display(fmt = "The value for the header '{}' could not be processed", header)]
    InvalidHeaderValue {
        /// The name of the malformed header
        header: &'static str,
    },

    /// `401 UNAUTHORIZED`
    ///
    /// Error code 40100
    #[display(
        fmt = "The server could not verify that you are authorized to access the URL requested. You either supplied the wrong credentials \
               (e.g. a bad password) or your browser doesn't understand how to supply the credentials required."
    )]
    Unauthorized,

    /// `403 FORBIDDEN`
    ///
    /// Error Code `40300`
    #[display(
        fmt = "You don't have the permission to access the requested resource. It is either read-protected or not readable by the server."
    )]
    Forbidden,

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
        required: Permission,
    },

    /// `404 NOT FOUND`
    ///
    /// Error Code `40400`
    #[display(
        fmt = "The requested URL was not found on the server. If you entered the URL manually please check your spelling and try again."
    )]
    NotFound,

    /// `405 METHOD NOT ALLOWED`
    ///
    /// Error Code `40500`
    #[display(fmt = "The method is not allowed for the requested URL.")]
    MethodNotAllowed,

    /// `409 CONFLICT`. This variant is returned if a `DELETE` or `PATCH` request is being handled,
    /// but the database transaction the operation is being performed in get rolled back due to a
    /// concurrent modification.
    ///
    /// Error Code `40900`
    #[display(
        fmt = "A conflict happened while processing the request. The resource might have been modified while the request was being \
               processed."
    )]
    Conflict,

    /// `411 LENGTH REQUIRED`
    ///
    /// Error Code `41100`
    #[display(fmt = "A request with this methods requires a valid 'Content-Length' header")]
    LengthRequired,

    /// `412 PRECONDITION FAILED`. This variant is returned if a `DELETE` or `PATCH` request is
    /// made, but the provided `If-Match` header doesn't match the hash of the object currently
    /// in the database
    ///
    /// Error Code `41200`
    #[display(fmt = "The precondition on the request for the URL failed positive evaluation")]
    PreconditionFailed,

    /// `413 PAYLOAD TOO LARGE`
    ///
    /// Error Code `41300`
    #[display(fmt = "The data value transmitted exceeds the capacity limit.")]
    PayloadTooLarge,

    /// `415 UNSUPPORTED MEDIA TYPE`
    ///
    /// Error Code `41500`
    #[display(
        fmt = "The server does not support the media type transmitted in the request/no media type was specified. Expected one '{}'",
        expected
    )]
    UnsupportedMediaType {
        /// The expected media type for the request body
        expected: &'static str,
    },

    /// `422 UNPROCESSABLE ENTITY`
    ///
    /// Error Code `42200`
    #[display(fmt = "The request was well-formed but was unable to be followed due to semantic errors.")]
    UnprocessableEntity,

    /// `422 UNPRECESSABLE ENTITY` variant returned if the `limit` parameter provided for
    /// pagination is too large or too small
    ///
    /// Error Code `42207`
    #[display(fmt = "Invalid value for the 'limit' parameter. It must be between 1 and 100")]
    InvalidPaginationLimit,

    /// `422 UNPROCESSABLE ENTITY` variant
    ///
    /// Error Code `42222`
    #[display(fmt = "Invalid URL scheme. Only 'http' and 'https' are supported")]
    InvalidUrlScheme,

    /// `422 UNPROCESSABLE ENTITY` variant
    ///
    /// Error Code `42223`
    #[display(fmt = "The provided URL contains authentication information. For security reasons it has been rejected")]
    UrlAuthenticated,

    /// `422 UNPROCESSABLE ENTITY` variant
    ///
    /// Error Code `42225`
    #[display(
        fmt = "The given URL does not lead to a video. The URL format for the given host has to be '{}'",
        expected
    )]
    InvalidUrlFormat {
        /// A hint as to how the format is expected to look
        expected: &'static str,
    },

    /// `422 UNPROCESSABLE ENTITY` variant
    ///
    /// Error Code `42227`
    #[display(
        fmt = "The 'after' value provided for pagination is smaller than the 'before' value. This would result in an empty response is \
               most likely a bug"
    )]
    AfterSmallerBefore,

    /// `422 UNPROCESSABLE ENTITY` variant
    ///
    /// Error Code `42229`
    #[display(fmt = "Your request contains mutually exclusive fields. Please restrict yourself to one of them")]
    MutuallyExclusive,

    /// `428 PRECONDITION REQUIRED`
    ///
    /// Error Code `42800`
    #[display(fmt = "This request is required to be conditional; try using \"If-Match\"")]
    PreconditionRequired,

    /// `429 TOO MANY REQUESTS`
    ///
    /// Error Code `42900`
    #[display(fmt = "{} Try again in {:.2?}", message, remaining)]
    Ratelimited {
        #[serde(skip)]
        message: String,

        remaining: Duration,
    },

    /// `500 INTERNAL SERVER ERROR`
    #[display(
        fmt = "The server encountered an internal error and was unable to complete your request. Either the server is overloaded or there \
               is an error in the application. Please notify a server administrator and have them look at the server logs!"
    )]
    InternalServerError,

    /// `500 INTERNAL SERVER ERROR`
    ///
    /// Error Code `50003`
    #[display(
        fmt = "Internally, an invalid database access has been made. Please notify a server administrator and have them look at the \
               server logs!"
    )]
    DatabaseError,

    /// `500 INTERNAL SERVER ERROR` reported when postgres terminates a query due to hitting `statement_timeout`
    ///
    /// Error Code `50004`
    #[display(
        fmt = "Internally, a database query timed out. This could be due to high server load, or because of a logic error resulting in a deadlock. If this issue persists after retrying, please notify a server administrator!"
    )]
    QueryTimeout,

    /// `500 INTERNAL SERVER ERROR` variant returned if the server fails to acquire a database
    /// connection
    ///
    /// Error Code `50005`
    #[display(fmt = "Failed to retrieve connection to the database. The server might be temporarily overloaded.")]
    DatabaseConnectionError,

    /// `503 SERVICE UNAVAILABLE` variant returned by all non-GET (e.g. all possible mutating) requests if the server is in maintenance mode.
    ///
    /// Error Core `50301`
    #[display(fmt = "The website is currently in read-only maintenance mode.")]
    ReadOnlyMaintenance,
}

impl CoreError {
    pub fn internal_server_error(message: impl AsRef<str>) -> CoreError {
        error!("INTERNAL SERVER ERROR: {}", message.as_ref());

        CoreError::InternalServerError
    }
}

impl Error for CoreError {}

impl PointercrateError for CoreError {
    fn error_code(&self) -> u16 {
        match self {
            CoreError::BadRequest => 40000,
            CoreError::InvalidHeaderValue { .. } => 40002,
            CoreError::Unauthorized => 40100,
            CoreError::Forbidden => 40300,
            CoreError::MissingPermissions { .. } => 40301,
            CoreError::NotFound => 40400,
            CoreError::MethodNotAllowed => 40500,
            CoreError::Conflict => 40900,
            CoreError::LengthRequired => 41200,
            CoreError::PreconditionFailed => 41200,
            CoreError::PayloadTooLarge => 41300,
            CoreError::UnsupportedMediaType { .. } => 41500,
            CoreError::UnprocessableEntity => 42200,
            CoreError::InvalidPaginationLimit => 42207,
            CoreError::InvalidUrlScheme => 42222,
            CoreError::UrlAuthenticated => 42223,
            CoreError::InvalidUrlFormat { .. } => 42225,
            CoreError::AfterSmallerBefore => 42227,
            CoreError::MutuallyExclusive => 42229,
            CoreError::PreconditionRequired => 42800,
            CoreError::Ratelimited { .. } => 42900,
            CoreError::InternalServerError { .. } => 50000,
            CoreError::DatabaseError => 50003,
            CoreError::QueryTimeout => 50004,
            CoreError::DatabaseConnectionError => 50005,
            CoreError::ReadOnlyMaintenance => 50301,
        }
    }
}

impl From<sqlx::Error> for CoreError {
    fn from(error: sqlx::Error) -> Self {
        error!("Database error: {:?}. Backtrace:\n {}", error, std::backtrace::Backtrace::capture());

        match error {
            sqlx::Error::Database(err) if err.code().as_deref() == Some("57014") => CoreError::QueryTimeout,
            sqlx::Error::PoolClosed | sqlx::Error::PoolTimedOut => CoreError::DatabaseConnectionError,
            _ => CoreError::DatabaseError,
        }
    }
}
