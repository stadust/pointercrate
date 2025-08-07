use crate::{localization::tr, permission::Permission, trp};
use log::error;
use serde::Serialize;
use std::{error::Error, fmt::Display, time::Duration};

pub type Result<T> = std::result::Result<T, CoreError>;

pub trait PointercrateError: Error + Serialize + From<CoreError> {
    fn error_code(&self) -> u16;
    fn status_code(&self) -> u16 {
        self.error_code() / 100
    }
}

#[derive(Serialize, Debug, Eq, PartialEq, Clone)]
#[serde(untagged)]
pub enum CoreError {
    /// Generic `400 BAD REQUEST` error
    ///
    /// Error Code `40000`
    BadRequest,

    /// `400 BAD REQUEST' error returned when a header value was malformed
    ///
    /// Error Code `40002`
    InvalidHeaderValue {
        /// The name of the malformed header
        header: &'static str,
    },

    /// `401 UNAUTHORIZED`
    ///
    /// Error code 40100
    Unauthorized,

    /// `403 FORBIDDEN`
    ///
    /// Error Code `40300`
    Forbidden,

    /// `403 FORBIDDEN` error that contains the permissions the client needs to have to perform the
    /// request
    ///
    /// Error Code `40301`
    MissingPermissions {
        /// The permissions required to perform the request
        required: Permission,
    },

    /// `404 NOT FOUND`
    ///
    /// Error Code `40400`
    NotFound,

    /// `405 METHOD NOT ALLOWED`
    ///
    /// Error Code `40500`
    MethodNotAllowed,

    /// `409 CONFLICT`. This variant is returned if a `DELETE` or `PATCH` request is being handled,
    /// but the database transaction the operation is being performed in get rolled back due to a
    /// concurrent modification.
    ///
    /// Error Code `40900`
    Conflict,

    /// `411 LENGTH REQUIRED`
    ///
    /// Error Code `41100`
    LengthRequired,

    /// `412 PRECONDITION FAILED`. This variant is returned if a `DELETE` or `PATCH` request is
    /// made, but the provided `If-Match` header doesn't match the hash of the object currently
    /// in the database
    ///
    /// Error Code `41200`
    PreconditionFailed,

    /// `413 PAYLOAD TOO LARGE`
    ///
    /// Error Code `41300`
    PayloadTooLarge,

    /// `415 UNSUPPORTED MEDIA TYPE`
    ///
    /// Error Code `41500`
    UnsupportedMediaType {
        /// The expected media type for the request body
        expected: &'static str,
    },

    /// `422 UNPROCESSABLE ENTITY`
    ///
    /// Error Code `42200`
    UnprocessableEntity,

    /// `422 UNPRECESSABLE ENTITY` variant returned if the `limit` parameter provided for
    /// pagination is too large or too small
    ///
    /// Error Code `42207`
    InvalidPaginationLimit,

    /// `422 UNPROCESSABLE ENTITY` variant
    ///
    /// Error Code `42222`
    InvalidUrlScheme,

    /// `422 UNPROCESSABLE ENTITY` variant
    ///
    /// Error Code `42223`
    UrlAuthenticated,

    /// `422 UNPROCESSABLE ENTITY` variant
    ///
    /// Error Code `42225`
    InvalidUrlFormat {
        /// A hint as to how the format is expected to look
        expected: &'static str,
    },

    /// `422 UNPROCESSABLE ENTITY` variant
    ///
    /// Error Code `42227`
    AfterSmallerBefore,

    /// `422 UNPROCESSABLE ENTITY` variant
    ///
    /// Error Code `42229`
    MutuallyExclusive,

    /// `428 PRECONDITION REQUIRED`
    ///
    /// Error Code `42800`
    PreconditionRequired,

    /// `429 TOO MANY REQUESTS`
    ///
    /// Error Code `42900`
    Ratelimited {
        #[serde(skip)]
        message: String,

        remaining: Duration,
    },

    /// `500 INTERNAL SERVER ERROR`
    InternalServerError,

    /// `500 INTERNAL SERVER ERROR`
    ///
    /// Error Code `50003`
    DatabaseError,

    /// `500 INTERNAL SERVER ERROR` reported when postgres terminates a query due to hitting `statement_timeout`
    ///
    /// Error Code `50004`
    QueryTimeout,

    /// `500 INTERNAL SERVER ERROR` variant returned if the server fails to acquire a database
    /// connection
    ///
    /// Error Code `50005`
    DatabaseConnectionError,

    /// `503 SERVICE UNAVAILABLE` variant returned by all non-GET (e.g. all possible mutating) requests if the server is in maintenance mode.
    ///
    /// Error Core `50301`
    ReadOnlyMaintenance,
}

impl CoreError {
    pub fn internal_server_error(message: impl AsRef<str>) -> CoreError {
        log_internal_server_error(message);

        CoreError::InternalServerError
    }
}

pub fn log_internal_server_error(message: impl AsRef<str>) {
    error!(
        "INTERNAL SERVER ERROR: {}. Backtrace:\n {}",
        message.as_ref(),
        std::backtrace::Backtrace::capture()
    );
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
            CoreError::LengthRequired => 41100,
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
            CoreError::InternalServerError => 50000,
            CoreError::DatabaseError => 50003,
            CoreError::QueryTimeout => 50004,
            CoreError::DatabaseConnectionError => 50005,
            CoreError::ReadOnlyMaintenance => 50301,
        }
    }
}

impl Display for CoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                CoreError::BadRequest => tr("error-core-badrequest"),
                CoreError::InvalidHeaderValue { header } => trp!("error-core-badrequest", "header" = header),
                CoreError::Unauthorized => tr("error-core-unauthorized"),
                CoreError::Forbidden => tr("error-core-badrequest"),
                CoreError::MissingPermissions { required } =>
                    trp!("error-core-missingpermissions", "required-permission" = tr(required.text_id())),
                CoreError::NotFound => tr("error-core-notfound"),
                CoreError::MethodNotAllowed => tr("error-core-methodnotallowed"),
                CoreError::Conflict => tr("error-core-conflict"),
                CoreError::LengthRequired => tr("error-core-lengthrequired"),
                CoreError::PreconditionFailed => tr("error-core-preconditionfailed"),
                CoreError::PayloadTooLarge => tr("error-core-payloadtoolarge"),
                CoreError::UnsupportedMediaType { expected } => trp!("error-core-unsupportedmediatype", "expected-type" = expected),
                CoreError::UnprocessableEntity => tr("error-core-unprocessableentity"),
                CoreError::InvalidPaginationLimit => tr("error-core-invalidpaginationlimit"),
                CoreError::InvalidUrlScheme => tr("error-core-invalidurlscheme"),
                CoreError::UrlAuthenticated => tr("error-core-urlauthenticated"),
                CoreError::InvalidUrlFormat { expected } => trp!("error-core-invalidurlformat", "expected-format" = expected),
                CoreError::AfterSmallerBefore => tr("error-core-aftersmallerbefore"),
                CoreError::MutuallyExclusive => tr("error-core-mutuallyexclusive"),
                CoreError::PreconditionRequired => tr("error-core-preconditionrequired"),
                CoreError::Ratelimited { message, remaining } => trp!(
                    "error-core-ratelimited",
                    "message" = message,
                    "remaining-duration" = format!("{:.2?}", remaining)
                ),
                CoreError::InternalServerError => tr("error-core-internalservererror"),
                CoreError::DatabaseError => tr("error-core-databaseerror"),
                CoreError::QueryTimeout => tr("error-core-querytimeout"),
                CoreError::DatabaseConnectionError => tr("error-core-databaseconnectionerror"),
                CoreError::ReadOnlyMaintenance => tr("error-core-readonlymaintenance"),
            }
        )
    }
}

impl From<sqlx::Error> for CoreError {
    fn from(error: sqlx::Error) -> Self {
        /*
         When creating resources that are subject to a unique constraint, there will
         always be a TOCTOU-style race condition. For example, creating new account
         has a check along the lines of "if username exists in database, return UsernameTaken error,
         else do insert into db". Check and insert are different queries, so concurrent creation
         of an account with this name is possible (at which point successful creation comes down to
         which connection commit()s first). These are not really internal server errors, so don't
         log and report them as such. HTTP 409 CONFLICT seems like the most appropriate response
         here.
        */
        if let sqlx::Error::Database(ref err) = error {
            if err.kind() == sqlx::error::ErrorKind::UniqueViolation {
                return CoreError::Conflict;
            }
        }

        log_internal_server_error(format!("Database error: {:?}", error));

        match error {
            sqlx::Error::Database(err) if err.code().as_deref() == Some("57014") => CoreError::QueryTimeout,
            sqlx::Error::PoolClosed | sqlx::Error::PoolTimedOut => CoreError::DatabaseConnectionError,
            _ => CoreError::DatabaseError,
        }
    }
}
