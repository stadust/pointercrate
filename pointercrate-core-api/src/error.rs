use crate::response::Page;
use log::{error, warn};
use pointercrate_core::error::PointercrateError;
use pointercrate_core_pages::error::ErrorFragment;
use rocket::{
    http::{MediaType, Status},
    response::Responder,
    serde::json::Json,
    Request, Response,
};
use serde::Serialize;
use serde_json::Value;

pub type Result<T> = std::result::Result<T, ErrorResponder>;

#[derive(Debug, Serialize)]
pub struct ErrorResponder {
    message: String,
    error_code: u16,
    data: Value,
}

impl<'r> Responder<'r, 'static> for ErrorResponder {
    fn respond_to(self, request: &'r Request<'_>) -> rocket::response::Result<'static> {
        let accept = match request.accept() {
            None => {
                warn!("No ACCEPT header set, assuming application/json");

                MediaType::JSON
            },
            Some(accept) => accept.preferred().0.clone(), // ?????
        };

        let status = Status::from_code(self.error_code / 100).unwrap_or(Status::InternalServerError);

        if status.code >= 500 {
            error!("Encountered an internal server error: {:?}", self);
        }

        if accept == MediaType::HTML {
            Response::build_from(
                Page(ErrorFragment {
                    status: self.error_code / 100,
                    reason: status.reason_lossy().to_string(),
                    message: self.message,
                })
                .respond_to(request)?,
            )
            .status(status)
            .ok()
        } else {
            Response::build_from(Json(self).respond_to(request)?).status(status).ok()
        }
    }
}

impl<E: PointercrateError> From<E> for ErrorResponder {
    fn from(error: E) -> Self {
        ErrorResponder {
            message: error.to_string(),
            error_code: error.error_code(),
            data: serde_json::to_value(error).expect("failed to serialize error to json"),
        }
    }
}
