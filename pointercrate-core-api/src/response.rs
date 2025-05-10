use crate::{
    etag::Tagged,
    preferences::{ClientPreferences, PreferenceManager},
};
use maud::{html, PreEscaped, DOCTYPE};
use pointercrate_core::etag::Taggable;
use pointercrate_core_pages::{
    head::{Head, HeadLike},
    localization::LocalizationConfiguration,
    PageConfiguration, PageFragment,
};
use rocket::{
    http::{ContentType, Header, Status},
    response::Responder,
    serde::json::Json,
    Request, Response,
};
use serde::Serialize;
use std::{borrow::Cow, io::Cursor};

pub struct Page(PageFragment, Vec<&'static str>);

impl Page {
    pub fn new(fragment: impl Into<PageFragment>, resources: Vec<&'static str>) -> Self {
        Page(fragment.into(), resources)
    }
}

impl HeadLike for Page {
    fn head_mut(&mut self) -> &mut Head {
        self.0.head_mut()
    }
}

impl<'r, 'o: 'r> Responder<'r, 'o> for Page {
    fn respond_to(self, request: &'r Request<'_>) -> rocket::response::Result<'o> {
        let page_config = request.rocket().state::<PageConfiguration>().ok_or(Status::InternalServerError)?;
        let preference_manager = request.rocket().state::<PreferenceManager>().ok_or(Status::InternalServerError)?;
        let localization_config = request
            .rocket()
            .state::<LocalizationConfiguration>()
            .ok_or(Status::InternalServerError)?;

        let preferences = ClientPreferences::from_cookies(request.cookies(), preference_manager);
        let locale_set = localization_config.set_by_uri(request.uri().path().segments().collect());
        let locale = locale_set
            .by_code(preferences.get::<String>(locale_set.cookie))
            .ok_or(Status::BadRequest)?;

        let fragment = self.0;

        // there has to be a better way to do this..
        // this loads an array of ftl resources this page requires, accessible
        // by the frontend js
        let mut resource_array_pushes = String::new();
        self.1
            .iter()
            .for_each(|resource| resource_array_pushes += &format!(r#"window.ftlResources.push("{}");"#, resource));

        let rendered_fragment = html! {
            (DOCTYPE)
            html lang=(locale.iso_code) prefix="og: http://opg.me/ns#" {
                head {
                    (page_config.head)
                    (fragment.head)
                    (PreEscaped(format!(r#"
                    <script>
                        window.ftlResources = [];
                        {}
                    </script>
                    "#, resource_array_pushes)))
                }
                body {
                    div.content {
                        (page_config.nav_bar.render(locale, locale_set))
                        (fragment.body)
                        div #bg {}
                    }
                    (page_config.footer)
                }
            }
        }
        .0;

        Response::build()
            .status(Status::Ok)
            .header(ContentType::HTML)
            .sized_body(rendered_fragment.len(), Cursor::new(rendered_fragment))
            .ok()
    }
}

pub struct Response2<T> {
    content: T,
    status: Status,
    headers: Vec<Header<'static>>,
}

impl<T: Serialize> Response2<Json<T>> {
    pub fn json(content: T) -> Self {
        Response2::new(Json(content))
    }
}

impl<T: Taggable> Response2<Tagged<T>> {
    pub fn tagged(content: T) -> Self {
        Response2::new(Tagged(content))
    }
}

impl<T> Response2<T> {
    pub fn new(content: T) -> Self {
        Response2 {
            content,
            status: Status::Ok,
            headers: vec![],
        }
    }

    pub fn with_header(mut self, name: &'static str, value: impl Into<Cow<'static, str>>) -> Self {
        self.headers.push(Header::new(name, value));
        self
    }

    pub fn status(mut self, status: Status) -> Self {
        self.status = status;
        self
    }
}

impl<'r, 'o: 'r, T: Responder<'r, 'o>> Responder<'r, 'o> for Response2<T> {
    fn respond_to(self, request: &'r Request<'_>) -> rocket::response::Result<'o> {
        let mut response_builder = Response::build_from(self.content.respond_to(request)?);
        response_builder.status(self.status);

        for header in self.headers {
            response_builder.header(header);
        }

        response_builder.ok()
    }
}
