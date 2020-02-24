//! Some utils for pagination and patch

use serde::{de::Error, Deserialize, Deserializer};

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
