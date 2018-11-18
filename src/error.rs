//! Moduling containing the [`PointercrateError`] enum.

use actix_web::{
    error::JsonPayloadError,
    http::{Method, StatusCode},
    HttpResponse, ResponseError,
};
use crate::model::{record::RecordStatus, user::PermissionsSet};
use diesel::result::Error;
use failure::Fail;
use log::error;
use serde_derive::Serialize;
use serde_json::json;

#[derive(Debug, Fail, Serialize)]
#[serde(untagged)]
pub enum PointercrateError {
    /// Generic `400 BAD REQUEST` error
    ///
    /// Error Code `40000`
    #[fail(
        display = "The browser (or proxy) sent a request that this server could not understand."
    )]
    GenericBadRequest,

    /// `400 BAD REQUEST` error with a message
    ///
    /// Error Code `40000`
    #[fail(display = "{}", message)]
    BadRequest {
        #[serde(skip)]
        message: String,
    },

    /// `400 BAD REQUEST' error returned when a header value was malformed
    ///
    /// Error Code `40002`
    #[fail(display = "The value for the header {} could not be processed", header)]
    InvalidHeaderValue {
        /// The name of the malformed header
        header: &'static str,
    },

    /// `401 UNAUTHORIZED`
    ///
    /// Erro code 40100
    #[fail(
        display = "The server could not verify that you are authorized to access the URL requested. You either supplied the wrong credentials (e.g. a bad password) or your browser doesn't understand how to supply the credentials required."
    )]
    Unauthorized,

    /// `403 FORBIDDEN`
    ///
    /// Error Code `40300`
    #[fail(
        display = "You don't have the permission to access the requested resource. It is either read-protected or not readable by the server."
    )]
    Forbidden,

    /// `403 FORBIDDEN` error that contains the permissions the client needs to have to perform the
    /// request
    ///
    /// Error Code `40301`
    #[fail(
        display = "You do not have the pointercrate permissions to perform this request. Required are: {}",
        required
    )]
    MissingPermissions {
        /// The permissions required to perform the request
        required: PermissionsSet,
    },

    /// `403 FORBIDDEN` error returned if someone with an IP-adress that's banned from submitting
    /// records tries to submit a record
    ///
    /// Error Code `40304`
    #[fail(display = "You are banned from submitting records to the demonlist!")]
    BannedFromSubmissions,

    /// `404 NOT FOUND`
    ///
    /// Error Code `40400`
    #[fail(
        display = "The requested URL was not found on the server. If you entered the URL manually please check your spelling and try again."
    )]
    NotFound,

    /// `404 NOT FOUND` error returned when a request references a non-existing model, e.g. submits
    /// a records for a demon that doesn't exist or tries to retrieve a player by an ID that
    /// isn't in use
    ///
    /// Error Code `40401`
    #[fail(display = "No '{}' identified by '{}' found!", model, identified_by)]
    ModelNotFound {
        /// The type of model attempted to retrieve
        #[serde(skip)]
        model: &'static str,

        /// The value by which the model the model was tried to be retrieved
        #[serde(skip)]
        identified_by: String,
    },

    // TODO: do something with allowed_methods
    /// `405 METHODE NOT ALLOWED`
    ///
    /// Error Code `40500`
    #[fail(display = "The method is not allowed for the requested URL.")]
    MethodNotAllowed {
        #[serde(skip)]
        allowed_methods: Vec<Method>,
    },

    /// `409 CONFLICT`. This variant is returned if a `DELETE` or `PATCH` request is being handled,
    /// but the database transaction the operation is being performed in get rolled back due to a
    /// concurrent modification.
    ///
    /// Error Code `40900`
    #[fail(
        display = "A conflict happened while processing the request. The resource might have been modified while the request was being processed."
    )]
    Conflict,

    /// `409 CONFLICT` error returned if a user tries to register with a name that's already taken
    ///
    /// Error Code `40902`
    #[fail(display = "The chosen username is already taken")]
    NameTaken,

    /// `411 LENGTH REQUIRED`
    ///
    /// Error Code `41100`
    #[fail(display = "A request with this methods requires a valid 'Content-Length' header")]
    LengthRequired,
    /// `412 PRECONDITION FAILED`. This variant is returned if a `DELETE` or `PATCH` request is
    /// made, but the provided `If-Match` header doesn't match the hash of the object currently
    /// in the database
    ///
    /// Error Code `41200`
    #[fail(display = "The precondition on the request for the URL failed positive evaluation")]
    PreconditionFailed,

    ///
    #[fail(display = "The data value transmitted exceeds the capacity limit.")]
    PayloadTooLarge,

    ///
    #[fail(
        display = "The server does not support the media type transmitted in the request/no media type was specified. Expected one '{}'",
        expected
    )]
    UnsupportedMediaType { expected: &'static str },

    ///
    #[fail(
        display = "The request was well-formed but was unable to be followed due to semeantic errors."
    )]
    UnprocessableEntity,

    ///
    #[fail(
        display = "Invalid display- or username! The name must be at least 3 characters long and not start/end with a space"
    )]
    InvalidUsername,

    ///
    #[fail(display = "Invalid password! The password must be at least 10 characters long")]
    InvalidPassword,

    ///
    #[fail(display = "Invalid URL scheme. Only 'http' and 'https' are supported")]
    InvalidUrlScheme,

    ///
    #[fail(
        display = "The provided URL contains authentication information. For security reasons it has been rejected"
    )]
    UrlAuthenticated,

    ///
    #[fail(
        display = "The given video host is not supported. Supported are 'youtube', 'vimeo', 'everyplay', 'twitch' and 'bilibili'"
    )]
    UnsupportedVideoHost,

    ///
    #[fail(
        display = "The given URL does not lead to a video. The URL format for the given host has to be '{}'",
        expected
    )]
    InvalidUrlFormat { expected: &'static str },

    ///
    #[fail(display = "Unexpected NULL value for field {}", field)]
    UnexpectedNull { field: &'static str },

    ///
    #[fail(display = "Record progress must lie between {} and 100%!", requirement)]
    InvalidProgress { requirement: i16 },

    ///
    #[fail(display = "This record has already been {}", status)]
    SubmissionExists { status: RecordStatus, existing: i32 },

    ///
    #[fail(display = "The given player is banned!")]
    PlayerBanned,

    ///
    #[fail(display = "You cannot submit records for legacy demons")]
    SubmitLegacy,

    ///
    #[fail(display = "Only 100% records can be submitted for the extended section of the list")]
    Non100Extended,

    ///
    #[fail(display = "This request is required to be conditional; try using \"If-Match\"")]
    PreconditionRequired,

    ///
    #[fail(
        display = "The server encountered an internal error and was unable to complete your request. Either the server is overloaded or there is an error in the application. Please notify a server administrator and have them look at the server logs!"
    )]
    InternalServerError,

    ///
    #[fail(
        display = "Internally, an invalid database access has been made. Please notify a server administrator and have them look at the server logs!"
    )]
    DatabaseError,

    ///
    #[fail(
        display = "Failed to retrieve connection to the database. The server might be temporarily overloaded."
    )]
    DatabaseConnectionError,
}

impl PointercrateError {
    pub fn database<E: Fail>(error: E) -> PointercrateError {
        error!(
            "Error while accessing database: {0}\t\tDebug output: {0:?}",
            error
        );

        PointercrateError::DatabaseError
    }

    pub fn internal<E: Fail>(error: E) -> PointercrateError {
        error!("Internal server error: {0}!\t\tDebug output: {0:?}", error);

        PointercrateError::InternalServerError
    }

    pub fn bad_request(message: &str) -> PointercrateError {
        PointercrateError::BadRequest {
            message: message.to_string(),
        }
    }

    pub fn error_code(&self) -> u16 {
        match self {
            PointercrateError::GenericBadRequest => 40000,
            PointercrateError::BadRequest { .. } => 40000,
            PointercrateError::InvalidHeaderValue { .. } => 40002,

            PointercrateError::Unauthorized => 40100,

            PointercrateError::Forbidden => 40300,
            PointercrateError::MissingPermissions { .. } => 40301,
            PointercrateError::BannedFromSubmissions => 40304,

            PointercrateError::NotFound => 40400,
            PointercrateError::ModelNotFound { .. } => 40401,

            PointercrateError::MethodNotAllowed { .. } => 40500,

            PointercrateError::Conflict => 40900,
            PointercrateError::NameTaken => 40902,

            PointercrateError::LengthRequired => 41100,

            PointercrateError::PreconditionFailed => 41200,

            PointercrateError::PayloadTooLarge => 41300,

            PointercrateError::UnsupportedMediaType { .. } => 41500,

            PointercrateError::UnprocessableEntity => 42200,
            PointercrateError::InvalidUsername => 42202,
            PointercrateError::InvalidPassword => 42204,
            PointercrateError::UnexpectedNull { .. } => 42211,
            PointercrateError::InvalidProgress { .. } => 42215,
            PointercrateError::SubmissionExists { .. } => 42217,
            PointercrateError::PlayerBanned => 42218,
            PointercrateError::SubmitLegacy => 42219,
            PointercrateError::Non100Extended => 42220,
            PointercrateError::InvalidUrlScheme => 42222,
            PointercrateError::UrlAuthenticated => 42223,
            PointercrateError::UnsupportedVideoHost => 42224,
            PointercrateError::InvalidUrlFormat { .. } => 42225,

            PointercrateError::PreconditionRequired => 42800,

            PointercrateError::InternalServerError => 50000,
            PointercrateError::DatabaseError => 50003,
            PointercrateError::DatabaseConnectionError => 50005,
        }
    }

    pub fn status_code(&self) -> StatusCode {
        let error_code = self.error_code();
        let status_code = error_code / 100;

        StatusCode::from_u16(status_code).unwrap()
    }
}

impl ResponseError for PointercrateError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).json(json!({
            "code": self.error_code(),
            "message": self.to_string(),
            "data": self
        }))
    }
}

impl From<JsonPayloadError> for PointercrateError {
    fn from(error: JsonPayloadError) -> Self {
        match error {
            JsonPayloadError::ContentType =>
                PointercrateError::UnsupportedMediaType {
                    expected: "application/json",
                },
            JsonPayloadError::Overflow => PointercrateError::PayloadTooLarge,
            JsonPayloadError::Payload(_) => PointercrateError::GenericBadRequest,
            JsonPayloadError::Deserialize(inner) =>
                PointercrateError::BadRequest {
                    message: inner.to_string(),
                },
        }
    }
}

impl From<Error> for PointercrateError {
    fn from(error: Error) -> Self {
        match error {
            Error::RollbackTransaction => PointercrateError::Conflict,
            err => PointercrateError::database(err),
        }
    }
}
