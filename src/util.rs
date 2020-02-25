//! Some utils for pagination and patch

use actix_web::HttpResponse;
use serde::{de::Error, Deserialize, Deserializer};

macro_rules! pagination_response {
    ($objects:expr, $pagination:expr, $min_id:expr, $max_id:expr, $before_field:ident, $after_field:ident, $($id_field:tt)*) => {{
        $pagination.$after_field = Some($min_id - 1);
        $pagination.$before_field = None;

        let mut rel = format!(
            "</api/v1/users/?{}>; rel=first",
            serde_urlencoded::to_string(&$pagination.0).unwrap()
        );

        $pagination.$after_field = None;
        $pagination.$before_field = Some($max_id + 1);

        rel.push_str(&format!(
            ",</api/v1/users/?{}>; rel=last",
            serde_urlencoded::to_string(&$pagination.0).unwrap()
        ));

        if !$objects.is_empty() {
            let first = $objects.first().unwrap().$($id_field)*;
            let last = $objects.last().unwrap().$($id_field)*;

            if first != $min_id {
                $pagination.$before_field = Some(first);
                $pagination.$after_field = None;

                rel.push_str(&format!(
                    ",</api/v1/users/?{}>; rel=prev",
                    serde_urlencoded::to_string(&$pagination.0).unwrap()
                ));
            }
            if last != $max_id {
                $pagination.$after_field = Some(last);
                $pagination.$before_field = None;

                rel.push_str(&format!(
                    ",</api/v1/users/?{}>; rel=next",
                    serde_urlencoded::to_string(&$pagination.0).unwrap()
                ));
            }
        }

        Ok(HttpResponse::Ok().header("Links", rel).json($objects))
    }};
}

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
}

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

macro_rules! __op {
    ($table:ident:: $column:ident = $value:expr) => {
        $table::$column.eq($value)
    };
    ($table:ident:: $column:ident < $value:expr) => {
        $table::$column.lt($value)
    };
    ($table:ident:: $column:ident > $value:expr) => {
        $table::$column.gt($value)
    };
    ($table:ident:: $column:ident <= $value:expr) => {
        $table::$column.le($value)
    };
    ($table:ident:: $column:ident >= $value:expr) => {
        $table::$column.ge($value)
    };
}

macro_rules! filter {
    ($query: ident[$($table: ident . $column: ident $op: tt $value: expr),+] limit $limit: expr) => {{
        let mut conditions = Vec::new();
        let mut counter = 1;
        let mut values = Vec::new();
        $(
            if let Some(ref value) = $value {
                conditions.push(concat!(stringify!($table), ".", stringify!($column), stringify!($op), " $").to_string() + &counter.to_string());
                counter += 1;
                values.push(value.to_string());
            }
        )+
        $query += &conditions.join(" AND ");
        $query += &format!("LIMIT ${}", counter);

        let mut query = sqlx::query(&$query);

        for value in values {
            query = query.bind(value);
        }

        query.bind($limit.unwrap_or(50).to_string())
    }};
}
