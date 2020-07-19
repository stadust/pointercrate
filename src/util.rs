//! Some utils for pagination and patch

use crate::error::PointercrateError;
use actix_web::{dev::HttpResponseBuilder, http::HeaderMap, HttpResponse};
use log::warn;
use mime::Mime;
use serde::{de::Error, Deserialize, Deserializer, Serialize};
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    str::FromStr,
};

macro_rules! pagination_response {
    ($endpoint: expr, $objects:expr, $pagination:expr, $min_id:expr, $max_id:expr, $before_field:ident, $after_field:ident, $($id_field:tt)*) => {{
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
                            $endpoint, serde_urlencoded::to_string(&$pagination.0).unwrap()
                        ));
                    }

                    if after.is_some() {
                        $pagination.$after_field = None;
                        $pagination.$before_field = Some(first);

                        rel.push_str(&format!(
                            ",<{}?{}>; rel=prev",
                            $endpoint, serde_urlencoded::to_string(&$pagination.0).unwrap()
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
                            $endpoint, serde_urlencoded::to_string(&$pagination.0).unwrap()
                        ));
                    }
                    $pagination.$after_field = Some(first);
                    $pagination.$before_field = None;

                    rel.push_str(&format!(
                        ",<{}?{}>; rel=next",
                        $endpoint, serde_urlencoded::to_string(&$pagination.0).unwrap()
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
            $endpoint, serde_urlencoded::to_string(&$pagination.0).unwrap()
        );

        $pagination.$after_field = None;
        $pagination.$before_field = Some($max_id + 1);

        links.push_str(&format!(
            ",<{}?{}>; rel=last",
            $endpoint, serde_urlencoded::to_string(&$pagination.0).unwrap()
        ));

        links.push_str(&rel);

        log::debug!("Links headers has value '{}'", links);

        Ok(HttpResponse::Ok().header("Links", links).json(&$objects))
    }};
}

pub fn header<'a>(request: &'a HeaderMap, header: &'static str) -> Result<Option<&'a str>, PointercrateError> {
    match request.get(header) {
        Some(value) =>
            value
                .to_str()
                .map_err(|_| PointercrateError::InvalidHeaderValue { header })
                .map(Some),
        None => Ok(None),
    }
}

pub fn parse_list_of_header_values<T: FromStr>(request: &HeaderMap, header_: &'static str) -> Result<Vec<T>, PointercrateError>
where
    T::Err: std::error::Error,
{
    let value = match header(request, header_)? {
        Some(value) => value,
        None => return Ok(Vec::new()),
    };

    value
        .split(',')
        .map(|value| value.parse())
        .collect::<Result<_, _>>()
        .map_err(|error| {
            warn!("Malformed '{}' header value in {}: {}", header_, value, error);

            PointercrateError::InvalidHeaderValue { header: header_ }
        })
}

pub fn preferred_mime_type(req: &HeaderMap) -> Result<Mime, PointercrateError> {
    let accepted: Vec<Mime> = parse_list_of_header_values(req, "Accept")?;

    let (preference, mime_type) = accepted
        .into_iter()
        .filter(|mime| {
            match (mime.type_(), mime.subtype()) {
                (mime::TEXT, mime::HTML) | (mime::APPLICATION, mime::JSON) => true,
                _ => false,
            }
        })
        .map(|mime| {
            (
                mime.get_param("q")
                    .map(|q| q.as_str().parse::<f32>().unwrap_or(-1.0))
                    .unwrap_or(1.0),
                mime,
            )
        })
        .max_by_key(|(q, _)| (q * 100.0) as i32)  // cannot compare floats dammit
        .unwrap_or((1.0, mime::TEXT_HTML));

    if preference < 0.0 || preference > 1.0 {
        Err(PointercrateError::InvalidHeaderValue { header: "Accept" })
    } else {
        Ok(mime_type)
    }
}
/*
macro_rules! header {
    ($req:expr, $header:expr) => {
        match $req.headers().get($header) {
            Some(value) =>
                Some(
                    value
                        .to_str()
                        .map_err(|_| PointercrateError::InvalidHeaderValue { header: $header })?,
                ),
            None => None,
        }
    };
}*/

#[allow(clippy::option_option)]
pub fn nullable<'de, T, D>(deserializer: D) -> std::result::Result<Option<Option<T>>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Ok(Some(Option::deserialize(deserializer)?))
}

pub fn non_nullable<'de, T, D>(deseralizer: D) -> std::result::Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    match Option::deserialize(deseralizer)? {
        None => Err(<D as Deserializer<'de>>::Error::custom("null value on non-nullable field")),
        some => Ok(some),
    }
}

pub trait HttpResponseBuilderExt {
    fn etag<H: Hash>(&mut self, obj: &H) -> &mut Self;
    fn json_with_etag<H: Serialize + Hash>(&mut self, obj: &H) -> HttpResponse;
}

impl HttpResponseBuilderExt for HttpResponseBuilder {
    fn etag<H: Hash>(&mut self, obj: &H) -> &mut Self {
        let mut hasher = DefaultHasher::new();
        obj.hash(&mut hasher);
        self.header("ETag", hasher.finish().to_string())
    }

    fn json_with_etag<H: Serialize + Hash>(&mut self, obj: &H) -> HttpResponse {
        self.etag(obj).json(serde_json::json!({ "data": obj }))
    }
}
