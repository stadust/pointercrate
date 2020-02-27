use crate::error::PointercrateError;
use actix_web::{http::Method, HttpRequest, HttpResponse, ResponseError};

pub mod auth;
pub mod demonlist;
pub mod user;

pub fn handle_404_or_405(request: HttpRequest) -> HttpResponse {
    let path = request.path();

    if !path.ends_with('/') && request.method() == Method::GET {
        return HttpResponse::Found().header("Location", format!("{}/", path)).finish()
    }

    if request.resource_map().has_resource(request.path()) {
        return PointercrateError::MethodNotAllowed.dynamic(request.headers()).error_response()
    }

    PointercrateError::NotFound.dynamic(request.headers()).error_response()
}
