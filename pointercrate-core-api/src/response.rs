use crate::etag::Tagged;
use maud::{html, DOCTYPE};
use pointercrate_core::{
    etag::Taggable,
    pagination::{Pagination, PaginationParameters},
};
use pointercrate_core_pages::{
    head::{Head, HeadLike},
    PageConfiguration, PageFragment,
};
use rocket::{
    http::{ContentType, Header, Status},
    response::Responder,
    serde::json::Json,
    Request, Response,
};
use serde::Serialize;
use std::{borrow::Cow, collections::BTreeMap, io::Cursor};

pub struct Page(PageFragment);

impl Page {
    pub fn new(fragment: impl Into<PageFragment>) -> Self {
        Page(fragment.into())
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

        let fragment = self.0;

        let rendered_fragment = html! {
            (DOCTYPE)
            html lang="en" prefix="og: http://opg.me/ns#" {
                head {
                    (page_config.head)
                    (fragment.head)
                }
                body style="z-index:-10" {
                    // target this element to get background image
                    div style={"width: 100%;height: 100%;position: fixed;top: 0;left: 0;background-size: cover;background-repeat: repeat-y;pointer-events: none; z-index:-1"} {}

                    (page_config.nav_bar)
                    (fragment.body)
                    (page_config.footer)
                }
            }
        }.0;

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

pub fn pagination_response<P, T, F>(
    endpoint: &'static str, mut objects: Vec<T>, paginate: P, min_id: i32, max_id: i32, id_func: F,
) -> Response2<Json<Vec<T>>>
where
    F: Fn(&T) -> i32,
    P: Pagination,
    T: Serialize,
{
    let parameters = paginate.parameters();
    // Use a BTreeMap so that we retain insertion order
    let mut rel = BTreeMap::new();

    rel.insert(
        "first",
        paginate.with_parameters(PaginationParameters {
            before: None,
            after: Some(min_id - 1),
            ..parameters
        }),
    );
    rel.insert(
        "last",
        paginate.with_parameters(PaginationParameters {
            before: Some(max_id + 1),
            after: None,
            ..parameters
        }),
    );

    let limit = parameters.limit as usize;
    let next_page_exists = objects.len() > limit;

    if !objects.is_empty() {
        if next_page_exists {
            objects.pop(); // remove the things from then next page
        }

        let last_id = id_func(objects.last().unwrap());
        let first_id = id_func(objects.first().unwrap());

        match (parameters.before, parameters.after) {
            (None, after) => {
                // no 'before' value set.
                // if 'after' is none, we're on the first page, otherwise we have ot generate a 'prev' link

                if next_page_exists {
                    rel.insert(
                        "next",
                        paginate.with_parameters(PaginationParameters {
                            before: None,
                            after: Some(last_id),
                            ..parameters
                        }),
                    );
                }

                if after.is_some() {
                    rel.insert(
                        "prev",
                        paginate.with_parameters(PaginationParameters {
                            before: Some(first_id),
                            after: None,
                            ..parameters
                        }),
                    );
                }
            },
            (Some(_), None) => {
                // A previous page exists. In this case, the page was retrieved using 'ORDER BY ... DESC' so we need to reverse list order!
                objects.reverse();

                // This means "first" and "last" are actually to opposite of what the variables are named.
                rel.insert(
                    "next",
                    paginate.with_parameters(PaginationParameters {
                        before: None,
                        after: Some(first_id),
                        ..parameters
                    }),
                );

                rel.insert(
                    "prev",
                    paginate.with_parameters(PaginationParameters {
                        before: Some(last_id),
                        after: None,
                        ..parameters
                    }),
                );
            },
            (Some(_before), Some(_after)) => {
                // We interpret this as that all objects _up to 'before'_ are supposed to be paginated.
                // This means we keep the 'before' value and handle the 'after' value just as above.
                // tODO: implement
            },
        }
    }

    // Would love to have Iterator::intersperse here
    let links = rel
        .into_iter()
        .map(|(tag, paginate)| format!("<{}?{}>; rel={}", endpoint, serde_urlencoded::to_string(paginate).unwrap(), tag))
        .collect::<Vec<_>>()
        .join(",");

    Response2::json(objects).with_header("Links", links)
}
