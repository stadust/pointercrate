use rocket::{
    form::{DataField, FromForm, Options, ValueField},
    http::Status,
    request::{FromRequest, Outcome},
    Request,
};
use serde::de::DeserializeOwned;
use std::fmt::Debug;

pub struct Query<T: DeserializeOwned>(pub T);

#[rocket::async_trait]
impl<'r, T: DeserializeOwned> FromRequest<'r> for Query<T> {
    type Error = serde_urlencoded::de::Error;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        match request.uri().query() {
            None => Outcome::Forward(()),
            Some(query) =>
                match serde_urlencoded::from_str(query.as_str()) {
                    Ok(t) => Outcome::Success(Query(t)),
                    Err(err) => Outcome::Failure((Status::BadRequest, err)),
                },
        }
    }
}
