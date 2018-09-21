use actix_web::{dev::FormConfig, error::UrlencodedError, Error, Form, FromRequest, HttpRequest, Responder};
use crate::{
    actor::demonlist::{ProcessSubmission, SubmitterByIp},
    error::PointercrateError,
    model::Submitter,
    PointercrateState,
};
use ipnetwork::IpNetwork;
use serde_derive::Deserialize;
use tokio::prelude::future::Future;

#[derive(Deserialize, Debug)]
pub struct Submission {
    pub progress: i16,
    pub player: String,
    pub demon: String,
    #[serde(default)]
    pub video: Option<String>,
    #[serde(rename = "check", default)]
    pub verify_only: bool,
}

fn submit_form_error_handler(error: UrlencodedError, _req: &HttpRequest<PointercrateState>) -> Error {
    match error {
        UrlencodedError::UnknownLength => PointercrateError::LengthRequired,
        UrlencodedError::ContentType =>
            PointercrateError::UnsupportedMediaType {
                expected: vec!["application/x-www-form-urlencoded"],
            },
        UrlencodedError::Overflow => PointercrateError::PayloadTooLarge,
        _ => PointercrateError::BadRequest,
    }.into()
}

pub fn submit(req: &HttpRequest<PointercrateState>) -> impl Responder {
    let mut form_config = FormConfig::default();
    form_config.error_handler(submit_form_error_handler);

    let _form = Form::<Submission>::from_request(req, &form_config)
        .and_then(|form: Form<Submission>| {
            let submission = form.into_inner();
            let remote_addr = req.extensions_mut().remove::<IpNetwork>().unwrap();

            req.state()
                .database
                .send(SubmitterByIp(remote_addr))
                .map_err(|_| PointercrateError::InternalServerError.into())
                .and_then(move |result| Ok((submission, result?)))
        }).and_then(|(submission, submitter): (Submission, Submitter)| {
            req.state()
                .database
                .send(ProcessSubmission(submission, submitter))
                .map_err(|_| PointercrateError::InternalServerError.into())
                .and_then(|result| Ok(result?))
        }); //TODO: generate JSON response
    "Hello World"
}
