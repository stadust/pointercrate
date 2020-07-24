//! Moduling containing the [`PointercrateError`] enum.

use crate::{
    model::demonlist::{demon::MinimalDemon, record::RecordStatus},
    permissions::Permissions,
    ratelimit::RatelimitScope,
    util::preferred_mime_type,
    view::{error::ErrorPage, Page},
};
use actix_web::{
    error::{JsonPayloadError, PathError, QueryPayloadError},
    http::{HeaderMap, Method, StatusCode},
    HttpResponse, ResponseError,
};
use derive_more::Display;
use log::error;
use serde::{
    ser::{SerializeSeq, Serializer},
    Serialize,
};
use serde_json::json;
use sqlx::Error;
use std::time::Duration;

// TODO: proper name
#[derive(Display, Debug)]
pub enum DynamicError {
    Json(JsonError),
    Html(HtmlError),
}

/// PointercrateError wrapper whose [`ResponseError`] impl generates a JSON response
#[derive(Display, Debug)]
pub struct HtmlError(pub PointercrateError);

/// PointercrateError wrapper whose [`ResponseError`] impl generates a HTML response
#[derive(Display, Debug, Serialize)]
#[serde(transparent)]
pub struct JsonError(pub PointercrateError);

impl std::error::Error for PointercrateError {}

#[derive(Debug, Display, Serialize, Clone, Eq, PartialEq)]
#[serde(untagged)]
pub enum PointercrateError {
    /// Generic `400 BAD REQUEST` error
    ///
    /// Error Code `40000`
    #[display(fmt = "The browser (or proxy) sent a request that this server could not understand.")]
    GenericBadRequest,

    /// `400 BAD REQUEST` error with a message
    ///
    /// Error Code `40000`
    #[display(fmt = "{}", message)]
    BadRequest {
        #[serde(skip)]
        message: String,
    },

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
    /// Erro code 40100
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
        required: Permissions,
    },

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

    /// `403 FORBIDDEN` error returned if someone with an IP-adress that's banned from submitting
    /// records tries to submit a record
    ///
    /// Error Code `40304`
    #[display(fmt = "You are banned from submitting records to the demonlist!")]
    BannedFromSubmissions,

    /// `404 NOT FOUND`
    ///
    /// Error Code `40400`
    #[display(
        fmt = "The requested URL was not found on the server. If you entered the URL manually please check your spelling and try again."
    )]
    NotFound,

    /// `404 NOT FOUND` error returned when a request references a non-existing model, e.g. submits
    /// a records for a demon that doesn't exist or tries to retrieve a player by an ID that
    /// isn't in use
    ///
    /// Error Code `40401`
    #[display(fmt = "No '{}' identified by '{}' found!", model, identified_by)]
    ModelNotFound {
        /// The type of model attempted to retrieve
        #[serde(skip)]
        model: &'static str,

        /// The value by which the model the model was tried to be retrieved
        #[serde(skip)]
        identified_by: String,
    },

    /// `405 METHOD NOT ALLOWED`
    ///
    /// Error Code `40500`
    #[display(fmt = "The method is not allowed for the requested URL.")]
    MethodNotAllowed, /* {
                          #[serde(serialize_with = "serialize_method")]
                          allowed_methods: Vec<Method>,
                      }*/

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

    /// `409 CONFLICT` error returned if a user tries to register with a name that's already taken
    ///
    /// Error Code `40902`
    #[display(fmt = "The chosen username is already taken")]
    NameTaken,

    /// `409 CONFLICT` error returned if a someone tries to add a demon with a name that's already
    /// taken by an existing demon
    ///
    /// Error Code: `40904`
    #[display(fmt = "A demon with the given name already exists at position {}", position)]
    DemonExists {
        /// The position of the existing [`Demon`]
        position: i16,
    },

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
    #[display(fmt = "The request was well-formed but was unable to be followed due to semeantic errors.")]
    UnprocessableEntity,

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

    /// `422 UNPRECESSABLE ENTITY` variant returned if the `limit` parameter provided for
    /// pagination is too large or too small
    ///
    /// Error Code `42207`
    #[display(fmt = "Invalid value for the 'limit' parameter. It must be between 1 and 100")]
    InvalidPaginationLimit,

    /// `422 UNPROCESSABLE ENTITY` variant
    ///
    /// Error Code `42211`
    #[display(fmt = "Unexpected NULL value for field {}", field)]
    UnexpectedNull { field: &'static str },

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
    #[display(fmt = "This record has already been {}", status)]
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
    /// Error Code `42224`
    #[display(fmt = "The given video host is not supported. Supported are 'youtube', 'vimeo', 'everyplay', 'twitch' and 'bilibili'")]
    UnsupportedVideoHost,

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
    /// Error Code `42226`
    #[display(fmt = "The given URL is no YouTube URL")]
    NotYouTube,

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
    /// Error Code `42228`
    #[display(fmt = "There are multiple demons with the given name")]
    DemonNameNotUnique { demons: Vec<MinimalDemon> },

    /// `422 UNPROCESSABLE ENTITY` variant
    ///
    /// Error Code `42229`
    #[display(fmt = "Your request contains mutually exclusive fields. Please restrict yourself to one of them")]
    MutuallyExclusive,

    /// `422 UNPROCESSABLE ENTITY` variant
    ///
    /// Error Code `42230`
    #[display(fmt = "Notes mustn't be empty!")]
    NoteEmpty,

    /// `428 PRECONDITION REQUIRED`
    ///
    /// Error Code `42800`
    #[display(fmt = "This request is required to be conditional; try using \"If-Match\"")]
    PreconditionRequired,

    /// `429 TOO MANY REQUESTS`
    ///
    /// Error Code `42900`
    #[display(fmt = "{}. Try again at in {:?}", scope, remaining)]
    Ratelimited {
        #[serde(skip)]
        scope: RatelimitScope,

        remaining: Duration,
    },

    /// `500 INTERNAL SERVER ERROR`
    #[display(
        fmt = "The server encountered an internal error and was unable to complete your request. Either the server is overloaded or there \
               is an error in the application. Please notify a server administrator and have them look at the server logs!"
    )]
    InternalServerError,

    #[display(fmt = "Internally, an object was supposed to be uniquely identified, yet more than one matching objects were found!")]
    Ambiguous,

    #[display(fmt = "The server internally entered an invalid state: {}", cause)]
    InvalidInternalStateError { cause: &'static str },

    /// `500 INTERNAL SERVER ERROR`
    ///
    /// Error Code `50003`
    #[display(
        fmt = "Internally, an invalid database access has been made. Please notify a server administrator and have them look at the \
               server logs!"
    )]
    DatabaseError,

    /// `500 INTERNAL SERVER ERROR` variant returned if the server fails to acquire a database
    /// connection
    ///
    /// Error Code `50005`
    #[display(fmt = "Failed to retrieve connection to the database. The server might be temporarily overloaded.")]
    DatabaseConnectionError,

    /// An error occured outside of pointercrate's application code (maybe during actix request
    /// handling or similar) and the error wasn't explicitly mapped to one of the other variants
    #[display(fmt = "{}", message)]
    Other {
        #[serde(skip)]
        status: StatusCode,

        #[serde(skip)]
        message: String,
    },
}

fn serialize_method<S>(methods: &[Method], serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut seq = serializer.serialize_seq(Some(methods.len()))?;
    for method in methods {
        seq.serialize_element(&method.to_string())?;
    }
    seq.end()
}

impl PointercrateError {
    pub fn error_code(&self) -> u16 {
        match self {
            PointercrateError::GenericBadRequest => 40000,
            PointercrateError::BadRequest { .. } => 40000,
            PointercrateError::InvalidHeaderValue { .. } => 40002,

            PointercrateError::Unauthorized => 40100,

            PointercrateError::Forbidden => 40300,
            PointercrateError::MissingPermissions { .. } => 40301,
            PointercrateError::DeleteSelf => 40302,
            PointercrateError::PatchSelf => 40303,
            PointercrateError::BannedFromSubmissions => 40304,

            PointercrateError::NotFound => 40400,
            PointercrateError::ModelNotFound { .. } => 40401,

            PointercrateError::MethodNotAllowed { .. } => 40500,

            PointercrateError::Conflict => 40900,
            PointercrateError::NameTaken => 40902,
            PointercrateError::DemonExists { .. } => 40904,

            PointercrateError::LengthRequired => 41100,

            PointercrateError::PreconditionFailed => 41200,

            PointercrateError::PayloadTooLarge => 41300,

            PointercrateError::UnsupportedMediaType { .. } => 41500,

            PointercrateError::UnprocessableEntity => 42200,
            PointercrateError::InvalidUsername => 42202,
            PointercrateError::InvalidPassword => 42204,
            PointercrateError::InvalidPaginationLimit => 42207,
            PointercrateError::UnexpectedNull { .. } => 42211,
            PointercrateError::InvalidRequirement => 42212,
            PointercrateError::InvalidPosition { .. } => 42213,
            PointercrateError::InvalidProgress { .. } => 42215,
            PointercrateError::SubmissionExists { .. } => 42217,
            PointercrateError::PlayerBanned => 42218,
            PointercrateError::SubmitLegacy => 42219,
            PointercrateError::Non100Extended => 42220,
            PointercrateError::InvalidUrlScheme => 42222,
            PointercrateError::UrlAuthenticated => 42223,
            PointercrateError::UnsupportedVideoHost => 42224,
            PointercrateError::InvalidUrlFormat { .. } => 42225,
            PointercrateError::NotYouTube => 42226,
            PointercrateError::AfterSmallerBefore => 42227,
            PointercrateError::DemonNameNotUnique { .. } => 42228,
            PointercrateError::MutuallyExclusive => 42229,
            PointercrateError::NoteEmpty => 42230,

            PointercrateError::PreconditionRequired => 42800,

            PointercrateError::Ratelimited { .. } => 42900,

            PointercrateError::InternalServerError => 50000,
            PointercrateError::InvalidInternalStateError { .. } => 50001,
            PointercrateError::Ambiguous => 50002,
            PointercrateError::DatabaseError => 50003,
            PointercrateError::DatabaseConnectionError => 50005,

            PointercrateError::Other { status, .. } => status.as_u16() * 100,
        }
    }

    pub fn status_code(&self) -> StatusCode {
        let error_code = self.error_code();
        let status_code = error_code / 100;

        StatusCode::from_u16(status_code).unwrap()
    }

    pub fn dynamic(self, request: &HeaderMap) -> DynamicError {
        let preferred = match preferred_mime_type(request) {
            Ok(pref) => pref,
            Err(error) => {
                // So being here means that the Accept header failed to parse.
                // We are going to assume that browsers set the headers correctly and that this an
                // API request. We therefore send out a json response.

                error!(
                    "Malformed 'Accept' header while trying to determine what format to use for error response. Defaulting to JSON. {}",
                    error
                );

                return DynamicError::Json(JsonError(self))
            },
        };

        match (preferred.type_(), preferred.subtype()) {
            (mime::TEXT, mime::HTML) => DynamicError::Html(HtmlError(self)),
            (mime::APPLICATION, mime::JSON) => DynamicError::Json(JsonError(self)),
            _ => unreachable!(),
        }
    }
}

impl From<Error> for JsonError {
    fn from(error: Error) -> Self {
        JsonError::from(PointercrateError::from(error))
    }
}

impl From<Error> for HtmlError {
    fn from(error: Error) -> Self {
        HtmlError::from(PointercrateError::from(error))
    }
}

impl From<PointercrateError> for JsonError {
    fn from(error: PointercrateError) -> Self {
        JsonError(error)
    }
}

impl From<PointercrateError> for HtmlError {
    fn from(error: PointercrateError) -> Self {
        HtmlError(error)
    }
}

impl ResponseError for DynamicError {
    fn status_code(&self) -> StatusCode {
        match self {
            DynamicError::Json(inner) => inner.status_code(),
            DynamicError::Html(inner) => inner.status_code(),
        }
    }

    fn error_response(&self) -> HttpResponse {
        match self {
            DynamicError::Json(inner) => inner.error_response(),
            DynamicError::Html(inner) => inner.error_response(),
        }
    }
}

impl ResponseError for JsonError {
    fn status_code(&self) -> StatusCode {
        self.0.status_code()
    }

    fn error_response(&self) -> HttpResponse {
        let mut response = HttpResponse::build(self.status_code());

        // FIXME: reimplement I guess
        /*if let PointercrateError::MethodNotAllowed { allowed_methods } = &self.0 {
            response.header("Allow", allowed_methods.iter().map(|m| m.to_string()).collect::<Vec<_>>().join(","));
        }*/

        response.json(json!({
            "code": self.0.error_code(),
            "message": self.0.to_string(),
            "data": self
        }))
    }
}

impl ResponseError for HtmlError {
    fn status_code(&self) -> StatusCode {
        self.0.status_code()
    }

    fn error_response(&self) -> HttpResponse {
        let html = ErrorPage::new(&self.0).render();

        HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .status(self.status_code())
            .body(html.0)
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

impl From<PathError> for PointercrateError {
    fn from(path_error: PathError) -> Self {
        match path_error {
            PathError::Deserialize(error) =>
                PointercrateError::BadRequest {
                    message: error.to_string(),
                },
        }
    }
}

impl From<QueryPayloadError> for PointercrateError {
    fn from(query_error: QueryPayloadError) -> Self {
        match query_error {
            QueryPayloadError::Deserialize(error) =>
                PointercrateError::BadRequest {
                    message: error.to_string(),
                },
        }
    }
}

impl From<Error> for PointercrateError {
    fn from(error: Error) -> Self {
        match error {
            Error::Database(database_error) => {
                error!(
                    "Database error: {}. Table: {:?}, column: {:?}, constraints: {:?}, hint: {:?}, details: {:?}",
                    database_error.message(),
                    database_error.table_name(),
                    database_error.column_name(),
                    database_error.constraint_name(),
                    database_error.hint(),
                    database_error.details()
                );

                PointercrateError::DatabaseError
            },
            Error::PoolClosed | Error::PoolTimedOut(_) => PointercrateError::DatabaseConnectionError,
            Error::ColumnNotFound(column) => {
                error!("Invalid access to column {}, which does not exist", column);

                PointercrateError::InternalServerError
            },
            Error::NotFound => {
                error!("Unhandled 'NotFound', this is a logic or data consistency error");

                PointercrateError::InternalServerError
            },
            Error::FoundMoreThanOne => PointercrateError::Ambiguous,
            _ => {
                error!("Database error: {:?}", error);

                PointercrateError::DatabaseError
            },
        }
    }
}
