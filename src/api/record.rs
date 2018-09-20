use actix_web::{dev::FormConfig, error::UrlencodedError, Error, Form, FromRequest, HttpRequest, Responder};
use actor::database::{ResolveSubmission, SubmitterByIp};
use error::PointercrateError;
use ipnetwork::IpNetwork;
use model::Submitter;
use tokio::prelude::future::Future;
use DemonlistState;

#[derive(Deserialize)]
pub struct Submission {
    pub progress: i16,
    pub player: String,
    pub demon: String,
    #[serde(default)]
    pub video: Option<String>,
    #[serde(rename = "check", default)]
    pub verify_only: bool,
}

fn submit_form_error_handler(error: UrlencodedError, req: &HttpRequest<DemonlistState>) -> Error {
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

pub fn submit(req: &HttpRequest<DemonlistState>) -> impl Responder {
    let mut form_config = FormConfig::default();
    form_config.error_handler(submit_form_error_handler);

    let form = Form::<Submission>::from_request(req, &form_config)
        .and_then(|form: Form<Submission>| {
            let submission = form.into_inner();
            let remote_addr = req.extensions_mut().remove::<IpNetwork>().unwrap();

            req.state()
                .database
                .send(SubmitterByIp(remote_addr))
                .map_err(|_| PointercrateError::InternalServerError.into())
                .and_then(|result| Ok((submission, result?)))
        }).and_then(|(submission, submitter): (Submission, Submitter)| {
            if submitter.banned() {
                return Err(PointercrateError::BannedFromSubmissions)?
            }

            Ok(submission)
        }).and_then(|submission| {
            req.state()
                .database
                .send(ResolveSubmission(submission))
                .map_err(|_| PointercrateError::InternalServerError.into())
                .and_then(move |result| result.map_err(Into::into))
        }).and_then(|(progress, player, demon, video, verify_only)| {
            let state = req.state();
            if player.banned() {}

            if demon.position() > state.extended_list_size {}

            if demon.position() > state.list_size && progress != 100 {}

            if progress > 100 || progress < demon.requirement() {}
            Ok(())
        });
    "Hello World"
}
