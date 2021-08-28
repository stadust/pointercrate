use crate::etag::Tagged;
use pointercrate_core::etag::Taggable;
use pointercrate_core_pages::{PageConfiguration, PageFragment};
use rocket::{
    http::{ContentType, Header, Status},
    response::Responder,
    serde::json::Json,
    Request, Response,
};
use serde::Serialize;
use std::{borrow::Cow, io::Cursor};

pub struct Page<T: PageFragment>(pub T);

impl<'r, 'o: 'r, T: PageFragment> Responder<'r, 'o> for Page<T> {
    fn respond_to(self, request: &'r Request<'_>) -> rocket::response::Result<'o> {
        let page_config = request.rocket().state::<PageConfiguration>().ok_or(Status::InternalServerError)?;
        let rendered_fragment = page_config.render_fragment(&self.0).0;

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

#[macro_export]
macro_rules! pagination_response {
    ($endpoint: expr, $objects:expr, $pagination:expr, $min_id:expr, $max_id:expr, $before_field:ident, $after_field:ident, $($id_field:tt)*) => {{
        use pointercrate_core_api::response::Response2;

        log::debug!("Received pagination request {:?}", $pagination);

        let mut rel = String::new();

        let limit = $pagination.limit.unwrap_or(50) as usize;
        let next_page_exists = $objects.len() > limit;

        if !$objects.is_empty() {
            if next_page_exists {
                log::debug!("A new page exists!");

                $objects.pop();  // remove the things from then next page
            }

            let last = $objects.last().unwrap().$($id_field)*;
            let first = $objects.first().unwrap().$($id_field)*;

            match ($pagination.$before_field, $pagination.$after_field) {
                (None, after) => {
                    log::debug!("No before value set, assuming result is correctly ordered!");

                    // no 'before' value set.
                    // if 'after' is none, we're on the first page, otherwise we have ot generate a 'prev' link

                    if next_page_exists {

                        $pagination.$after_field = Some(last);
                        $pagination.$before_field = None;

                        rel.push_str(&format!(
                            ",<{}?{}>; rel=next",
                            $endpoint, serde_urlencoded::to_string(&$pagination).unwrap()
                        ));
                    }

                    if after.is_some() {
                        $pagination.$after_field = None;
                        $pagination.$before_field = Some(first);

                        rel.push_str(&format!(
                            ",<{}?{}>; rel=prev",
                            $endpoint, serde_urlencoded::to_string(&$pagination).unwrap()
                        ));
                    }
                }
                (Some(_), None) => {
                    log::debug!("Before value set, assuming result is reverse ordered!");

                    // A previous page exists. This means "first" and "last" are actually to opposite of what the variables are named.
                    $pagination.$before_field = Some(last);
                    $pagination.$after_field = None;

                    // In this case, the page was retrieved using 'ORDER BY ... DESC' so we need to reverse list order!
                    $objects.reverse();

                    if next_page_exists {
                        rel.push_str(&format!(
                            ",<{}?{}>; rel=prev",
                            $endpoint, serde_urlencoded::to_string(&$pagination).unwrap()
                        ));
                    }
                    $pagination.$after_field = Some(first);
                    $pagination.$before_field = None;

                    rel.push_str(&format!(
                        ",<{}?{}>; rel=next",
                        $endpoint, serde_urlencoded::to_string(&$pagination).unwrap()
                    ));
                }
                (Some(_before), Some(_after)) => {
                    // We interpret this as that all objects _up to 'before'_ are supposed to be paginated.
                    // This means we keep the 'before' value and handle the 'after' value just as above.
                    // tODO: implement
                }
            }
        }

        $pagination.$after_field = Some($min_id - 1);
        $pagination.$before_field = None;

        let mut links = format!(
            "<{}?{}>; rel=first",
            $endpoint, serde_urlencoded::to_string(&$pagination).unwrap()
        );

        $pagination.$after_field = None;
        $pagination.$before_field = Some($max_id + 1);

        links.push_str(&format!(
            ",<{}?{}>; rel=last",
            $endpoint, serde_urlencoded::to_string(&$pagination).unwrap()
        ));

        links.push_str(&rel);

        log::debug!("Links headers has value '{}'", links);

        Ok(Response2::json($objects).with_header("Links", links))
    }};
}
