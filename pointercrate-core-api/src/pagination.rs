use std::collections::BTreeMap;

use pointercrate_core::{error::CoreError, pagination::{Pagination, PaginationParameters}};
use rocket::serde::json::Json;
use sqlx::PgConnection;

use crate::response::Response2;



pub async fn pagination_response<P: Pagination>(
    endpoint: &'static str, paginate: P, connection: &mut PgConnection
) -> Result<Response2<Json<Vec<P::Item>>>, CoreError>
{
    paginate.parameters().validate()?;

    let mut objects = paginate.page(&mut *connection).await?;

    let parameters = paginate.parameters();
    // Use a BTreeMap so that we retain insertion order
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

    let limit = parameters.limit as usize;
    let next_page_exists = objects.len() > limit;

    if !objects.is_empty() {
        if next_page_exists {
            objects.pop(); // remove the things from then next page
        }

        let last_id = P::id_of(objects.last().unwrap());
        let first_id = P::id_of(objects.first().unwrap());

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

    Ok(Response2::json(objects).with_header("Links", links))
}
