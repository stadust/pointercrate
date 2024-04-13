use std::collections::BTreeMap;

use pointercrate_core::{error::CoreError, pagination::{Pagination, PaginationParameters}};
use rocket::serde::json::Json;
use sqlx::PgConnection;

use crate::response::Response2;


pub async fn pagination_response<P: Pagination>(
    endpoint: &'static str, paginate: P, connection: &mut PgConnection
) -> Result<Response2<Json<Vec<P::Item>>>, CoreError>
{
    let parameters = paginate.parameters();

    parameters.validate()?;

    let (objects, context) = paginate.page(&mut *connection).await?;

    let mut rel = BTreeMap::new();

    if let Some((min_id, max_id)) = P::first_and_last(connection).await? {
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
    }

    if context.has_next() {
        let after = match objects.last() {
            Some(obj) => P::id_of(obj),
            None => {
                // If there exists a next page, but this page is empty, then 
                // we must have had a `before` value set (e.g. this is a page before the first object matching the pagination conditions).
                parameters.before.ok_or_else(|| {
                    CoreError::internal_server_error(format!(
                        "Empty page claims next page exists, yet `before` not set on current request. Caused by {:?}",
                        paginate
                    ))
                })? - 1
            },
        };

        rel.insert("next", paginate.with_parameters(PaginationParameters {
            // TODO: Figure out the case where both `before` and `after` are set
            // If `before` is set on this request, then we _could_ support one-way pagination up to `before` by preserving the "before" value here. 
            // Currently, this scenario cannot happen, as the documentation of `Pagination::page` we treat these pages as "standalone". 
            before: None,
            after: Some(after),
            ..parameters
        }));
    }

    if context.has_previous() {
        let before = match objects.first() {
            Some(obj) => P::id_of(obj),
            None => {
                parameters.after.ok_or_else(|| {
                    CoreError::internal_server_error(format!(
                        "Empty page claims previous page exists, yet `after` not set on current request. Caused by {:?}",
                        paginate
                    ))
                })? + 1
            },
        };

        rel.insert("prev", paginate.with_parameters(PaginationParameters {
            before: Some(before), 
            // Either this request had the `after` parameter set, in which case we definitely do not want to preserve it as our "before" variable above is either
            // the ID of the smallest object greater than `after`, or it is literally `after + 1`.
            after: None, 
            ..parameters
        }));
    };

    // Would love to have Iterator::intersperse here
    let links = rel
        .into_iter()
        .map(|(tag, paginate)| format!("<{}?{}>; rel={}", endpoint, serde_urlencoded::to_string(paginate).unwrap(), tag))
        .collect::<Vec<_>>()
        .join(",");

    Ok(Response2::json(objects).with_header("Links", links))
}
