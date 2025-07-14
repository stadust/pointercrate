use crate::localization::LOCALE_COOKIE_NAME;
use crate::preferences::{ClientPreferences, PreferenceManager};
use crate::response::Page;
use log::info;
use pointercrate_core::error::PointercrateError;
use pointercrate_core::localization::{LocaleConfiguration, LANGUAGE};
use pointercrate_core_pages::error::ErrorFragment;
use pointercrate_core_pages::PageFragment;
use rocket::futures;
use rocket::outcome::Outcome;
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
    #[serde(rename = "code")]
    error_code: u16,
    data: Value,
}

impl<'r> Responder<'r, 'static> for ErrorResponder {
    fn respond_to(self, request: &'r Request<'_>) -> rocket::response::Result<'static> {
        let accept = match request.accept() {
            None => {
                info!("No ACCEPT header set, assuming application/json");

                &MediaType::JSON
            },
            Some(accept) => &accept.preferred().0,
        };

        let status = Status::from_code(self.error_code / 100).unwrap_or(Status::InternalServerError);

        if *accept == MediaType::HTML {
            let preference_manager = request.rocket().state::<PreferenceManager>().ok_or(Status::InternalServerError)?;
            let preferences = ClientPreferences::from_cookies(request.cookies(), preference_manager);

            let language = preferences.get(LOCALE_COOKIE_NAME).ok_or(Status::InternalServerError)?;
            let lang_id = LocaleConfiguration::get().by_code(language);

            let fragment = futures::executor::block_on(async {
                LANGUAGE
                    .scope(lang_id.language, async {
                        PageFragment::from(ErrorFragment {
                            status: self.error_code / 100,
                            reason: status.reason_lossy().to_string(),
                            message: self.message,
                        })
                    })
                    .await
            });

            Response::build_from(Page::new(fragment).respond_to(request)?).status(status).ok()
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

/// A version of [`IntoOutcome`](rocket::outcome::IntoOutcome) specially crafted for [`PointercrateError`]s
pub trait IntoOutcome2<S, E> {
    fn into_outcome<F, E2: From<E>>(self) -> Outcome<S, (Status, E2), F>;
}

impl<S, E: PointercrateError> IntoOutcome2<S, E> for std::result::Result<S, E> {
    fn into_outcome<F, E2: From<E>>(self) -> Outcome<S, (Status, E2), F> {
        self.map(Outcome::Success).unwrap_or_else(|e| e.into_outcome())
    }
}

impl<S, E: PointercrateError> IntoOutcome2<S, E> for E {
    fn into_outcome<F, E2: From<E>>(self) -> Outcome<S, (Status, E2), F> {
        Outcome::Error((Status::new(self.status_code()), self.into()))
    }
}

#[macro_export]
macro_rules! tryo_result {
    ($result: expr) => {
        rocket::outcome::try_outcome!($crate::error::IntoOutcome2::into_outcome($result))
    };
}

#[macro_export]
macro_rules! tryo_state {
    ($request: expr, $typ: ty) => {
        $crate::tryo_result!($request
            .rocket()
            .state::<$typ>()
            .ok_or_else(|| pointercrate_core::error::CoreError::internal_server_error(format!(
                "Missing required state: '{}'",
                stringify!($typ)
            ))))
    };
}
