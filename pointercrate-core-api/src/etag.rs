use pointercrate_core::{error::CoreError, etag::Taggable};
use rocket::{
    http::{Method, Status},
    request::{FromRequest, Outcome},
    response::Responder,
    serde::json::Json,
    Request, Response,
};

pub struct Tagged<T: Taggable>(pub T);

pub struct Precondition(Vec<String> /* ensure private constructor for type level proof of header */);

impl Precondition {
    pub fn require_etag_match<T: Taggable>(&self, taggable: &T) -> Result<(), CoreError> {
        let patch_etag = taggable.patch_part().to_string();

        if self
            .0
            .iter()
            .filter_map(|if_match| if_match.split(';').next())
            .any(|e| e == &patch_etag)
        {
            Ok(())
        } else {
            Err(CoreError::PreconditionFailed)
        }
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Precondition {
    type Error = CoreError;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        match request.headers().get_one("if-match") {
            Some(if_match) => Outcome::Success(Precondition(if_match.split(',').map(ToString::to_string).collect())),
            None => Outcome::Failure((Status::PreconditionRequired, CoreError::PreconditionRequired)),
        }
    }
}

impl<'r, T: Taggable> Responder<'r, 'static> for Tagged<T> {
    fn respond_to(self, request: &'r Request<'_>) -> rocket::response::Result<'static> {
        let response_etag = self.0.etag_string();

        match request.method() {
            Method::Get =>
                if let Some(if_none_match) = request.headers().get_one("if-none-match") {
                    if if_none_match.contains(&response_etag) {
                        return Response::build().status(Status::NotModified).ok()
                    }
                },
            Method::Patch | Method::Delete =>
                if let Some(if_none_match) = request.headers().get_one("if-match") {
                    if if_none_match.contains(&response_etag) {
                        return Response::build().status(Status::NotModified).ok()
                    }
                },
            _ => (),
        }

        Response::build_from(Json(self.0).respond_to(request)?)
            .raw_header("etag", response_etag)
            .ok()
    }
}
