use std::collections::BTreeMap;

use pointercrate_core::{
    error::CoreError,
    pagination::{Paginatable, PaginationParameters, PaginationQuery},
};
use rocket::serde::json::Json;
use sqlx::PgConnection;

use crate::response::Response2;

#[derive(Debug)]
pub struct LinksBuilder {
    endpoint: &'static str,
    rels: BTreeMap<&'static str, PaginationParameters>,
}

impl LinksBuilder {
    pub fn new(endpoint: &'static str) -> Self {
        LinksBuilder {
            endpoint,
            rels: BTreeMap::new(),
        }
    }

    pub fn with_first(mut self, id_before_first: i32) -> Self {
        self.rels.insert(
            "first",
            PaginationParameters {
                after: Some(id_before_first),
                before: None,
                ..Default::default()
            },
        );
        self
    }

    pub fn with_last(mut self, id_after_last: i32) -> Self {
        self.rels.insert(
            "last",
            PaginationParameters {
                after: None,
                before: Some(id_after_last),
                ..Default::default()
            },
        );
        self
    }

    pub fn with_next(mut self, after: i32) -> Self {
        self.rels.insert(
            "next",
            PaginationParameters {
                after: Some(after),
                before: None,
                ..Default::default()
            },
        );
        self
    }

    pub fn with_previous(mut self, before: i32) -> Self {
        self.rels.insert(
            "prev",
            PaginationParameters {
                after: None,
                before: Some(before),
                ..Default::default()
            },
        );
        self
    }

    pub fn generate<P: PaginationQuery>(&self, base: &P) -> Result<String, CoreError> {
        let mut buf = String::new();
        let mut is_first = true;
        // The build functions set a default value for "limit" - copy the actual value from the given base here
        let limit = base.parameters().limit;

        for (rel, param) in &self.rels {
            if !is_first {
                buf.push_str(",");
            }
            is_first = false;

            let query_string =
                serde_urlencoded::to_string(base.with_parameters(PaginationParameters { limit, ..*param })).map_err(|err| {
                    CoreError::internal_server_error(format!(
                        "Failed to serialize pagination query string: {:?}. Base: {:?}, Builder: {:?}, Current Rel: {}",
                        err, base, self, rel
                    ))
                })?;

            buf += &format!("<{}?{}>; rel={}", self.endpoint, query_string, rel);
        }

        Ok(buf)
    }
}

pub async fn pagination_response<Q: PaginationQuery, P: Paginatable<Q>>(
    endpoint: &'static str, query: Q, connection: &mut PgConnection,
) -> Result<Response2<Json<Vec<P>>>, CoreError> {
    let parameters = query.parameters();

    parameters.validate()?;

    let (objects, context) = P::page(&query, &mut *connection).await?;

    let mut links = LinksBuilder::new(endpoint);

    if let Some((min_id, max_id)) = P::first_and_last(connection).await? {
        links = links.with_first(min_id - 1).with_last(max_id + 1);
    }

    if context.has_next() {
        let after = match objects.last() {
            Some(obj) => obj.pagination_id(),
            None => {
                // If there exists a next page, but this page is empty, then
                // we must have had a `before` value set (e.g. this is a page before the first object matching the pagination conditions).
                parameters.before.ok_or_else(|| {
                    CoreError::internal_server_error(format!(
                        "Empty page claims next page exists, yet `before` not set on current request. Caused by {:?}",
                        query
                    ))
                })? - 1
            },
        };

        // TODO: Figure out the case where both `before` and `after` are set
        // If `before` is set on this request, then we _could_ support one-way pagination up to `before` by preserving the "before" value here.
        // Currently, this scenario cannot happen, as the documentation of `Pagination::page` we treat these pages as "standalone".
        links = links.with_next(after);
    }

    if context.has_previous() {
        let before = match objects.first() {
            Some(obj) => obj.pagination_id(),
            None => {
                parameters.after.ok_or_else(|| {
                    CoreError::internal_server_error(format!(
                        "Empty page claims previous page exists, yet `after` not set on current request. Caused by {:?}",
                        query
                    ))
                })? + 1
            },
        };

        // Either this request had the `after` parameter set, in which case we definitely do not want to preserve it as our "before" variable above is either
        // the ID of the smallest object greater than `after`, or it is literally `after + 1`.
        links = links.with_previous(before);
    };

    Ok(Response2::json(objects).with_header("Links", links.generate(&query)?))
}

#[cfg(test)]
mod tests {
    use pointercrate_core::pagination::{PaginationParameters, PaginationQuery};
    use serde::Serialize;

    use super::LinksBuilder;

    #[derive(Debug, Default, Serialize)]
    struct DummyQuery(PaginationParameters);

    impl PaginationQuery for DummyQuery {
        fn parameters(&self) -> PaginationParameters {
            self.0
        }

        fn with_parameters(&self, parameters: PaginationParameters) -> Self {
            DummyQuery(parameters)
        }
    }

    #[test]
    fn test_links_builder() {
        let links_header = LinksBuilder::new("/dummies")
            .with_first(0)
            .with_last(1971)
            .with_next(2)
            .with_previous(100)
            .generate(&DummyQuery::default())
            .unwrap();

        assert_eq!(
            links_header,
            "</dummies?after=0>; rel=first,</dummies?before=1971>; rel=last,</dummies?after=2>; rel=next,</dummies?before=100>; rel=prev"
        );
    }
}
