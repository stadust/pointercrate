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
        $pagination.$after_field = Some($min_id - 1);
        $pagination.$before_field = None;

        let mut rel = format!(
            "<{}?{}>; rel=first",
            $endpoint, serde_urlencoded::to_string(&$pagination.0).unwrap()
        );

        $pagination.$after_field = None;
        $pagination.$before_field = Some($max_id + 1);

        rel.push_str(&format!(
            ",<{}?{}>; rel=last",
            $endpoint, serde_urlencoded::to_string(&$pagination.0).unwrap()
        ));

        if !$objects.is_empty() {
            let first = $objects.first().unwrap().$($id_field)*;
            let last = $objects.last().unwrap().$($id_field)*;

            if first != $min_id {
                $pagination.$before_field = Some(first);
                $pagination.$after_field = None;

                rel.push_str(&format!(
                    ",<{}?{}>; rel=prev",
                    $endpoint, serde_urlencoded::to_string(&$pagination.0).unwrap()
                ));
            }
            if last != $max_id {
                $pagination.$after_field = Some(last);
                $pagination.$before_field = None;

                rel.push_str(&format!(
                    ",<{}?{}>; rel=next",
                    $endpoint, serde_urlencoded::to_string(&$pagination.0).unwrap()
                ));
            }
        }

        Ok(HttpResponse::Ok().header("Links", rel).json($objects))
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
