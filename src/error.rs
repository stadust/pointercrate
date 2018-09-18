use actix_web::http::{Method, StatusCode};

#[derive(Debug, Fail)]
pub enum PointercrateError {
    #[fail(display = "The browser (or proxy) sent a request that this server could not understand.")]
    BadRequest,

    #[fail(
        display = "The server could not verify that you are authorized to access the URL requested. You either supplied the wrong credentials (e.g. a bad password) or your browser doesn't understand how to supply the credentials required."
    )]
    Unauthorized,

    #[fail(
        display = "You don't have the permission to access the requested resource. It is either read-protected or not readable by the server."
    )]
    Forbidden,

    #[fail(
        display = "The requested URL was not found on the server. If you entered the URL manually please check your spelling and try again."
    )]
    NotFound,

    #[fail(display = "The method is not allowed for the requested URL.")]
    MethodNotAllowed { allowed_methods: Vec<Method> },

    #[fail(
        display = "A conflict happened while processing the request. The resource might have been modified while the request was being processed."
    )]
    Conflict,

    #[fail(display = "The precondition on the request for the URL failed positive evaluation")]
    PreconditionFailed,

    #[fail(
        display = "The server does not support the media type transmitted in the request. Expected one of: {:?}",
        expected
    )]
    UnsupportedMediaType { expected: Vec<String> },

    #[fail(display = "The request was well-formed but was unable to be followed due to semeantic errors.")]
    UnprocessableEntity,

    #[fail(display = "This request is required to be conditional; try using \"If-Match\"")]
    PreconditionRequired,

    #[fail(
        display = "The server encountered an internal error and was unable to complete your requests. Either the server is overloaded or there is an error in the application."
    )]
    InternalServerError,
}

impl PointercrateError {
    pub fn error_code(&self) -> u16 {
        match self {
            PointercrateError::BadRequest => 40000,
            PointercrateError::Unauthorized => 40100,
            PointercrateError::Forbidden => 40300,
            PointercrateError::NotFound => 40400,
            PointercrateError::MethodNotAllowed { .. } => 40500,
            PointercrateError::Conflict => 40900,
            PointercrateError::PreconditionFailed => 41200,
            PointercrateError::UnsupportedMediaType { .. } => 41500,
            PointercrateError::UnprocessableEntity => 42200,
            PointercrateError::PreconditionRequired => 42800,
            PointercrateError::InternalServerError => 50000,
            _ => unimplemented!(),
        }
    }

    pub fn status_code(&self) -> StatusCode {
        let error_code = self.error_code();
        let status_code = error_code / 100;

        StatusCode::from_u16(status_code).unwrap()
    }
}
