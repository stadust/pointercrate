use std::fmt::Display;

use crate::{error::CoreError, util::non_nullable};
use serde::{de::Error, Deserialize, Serialize};
use sqlx::PgConnection;

/// The maximal number of entries that can be requested per page via the `limit` parameter.
pub const ENTRIES_PER_PAGE: i32 = 100;

/// The default number of entries returned per page if the `limit` parameter was omited.
/// 
/// Try not to directly rely on this constant, and instead use `PaginationParameters::default()`
pub const DEFAULT_ENTRIES_PER_PAGE: i32 = 50;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct PaginationParameters {
    #[serde(default, deserialize_with = "from_str_non_nullable")]
    pub before: Option<i32>,

    #[serde(default, deserialize_with = "from_str_non_nullable")]
    pub after: Option<i32>,

    #[serde(
        default = "default_limit",
        deserialize_with = "from_str",
        skip_serializing_if = "is_default_entries_per_page"
    )]
    pub limit: i32,
}

impl Default for PaginationParameters {
    fn default() -> Self {
        Self { before: None, after: None, limit: DEFAULT_ENTRIES_PER_PAGE }
    }
}

impl PaginationParameters {
    pub fn validate(&self) -> Result<(), CoreError> {
        if !(1..=ENTRIES_PER_PAGE).contains(&self.limit) {
            return Err(CoreError::InvalidPaginationLimit);
        }

        if let (Some(after), Some(before)) = (self.before, self.after) {
            if after < before {
                return Err(CoreError::AfterSmallerBefore);
            }
        }

        Ok(())
    }

    pub fn order(&self) -> &'static str {
        if self.after.is_none() && self.before.is_some() {
            "DESC"
        } else {
            "ASC"
        }
    }
}

#[allow(async_fn_in_trait)]
pub trait Pagination: Serialize {
    type Item: Serialize;

    fn parameters(&self) -> PaginationParameters;
    fn with_parameters(&self, parameters: PaginationParameters) -> Self;

    async fn page(&self, connection: &mut PgConnection) -> Result<Vec<Self::Item>, sqlx::Error>;

    async fn first_and_last(connection: &mut PgConnection) -> Result<Option<(i32, i32)>, sqlx::Error>;

    fn id_of(item: &Self::Item) -> i32;
}

#[macro_export]
macro_rules! first_and_last {
    ($table_name: expr, $id_column: expr) => {
        async fn first_and_last(connection: &mut PgConnection) -> std::result::Result<Option<(i32, i32)>, sqlx::Error> {
            let row = sqlx::query!("SELECT CAST(MIN(" + $id_column + ") AS INTEGER), CAST(MAX(" + $id_column + ") AS INTEGER) FROM " + $table_name)
                .fetch_one(connection)
                .await?;

            Ok(row.min.zip(row.max))
        }
    };
    ($table_name: expr) => {
        first_and_last!($table_name, "id");
    };
}

/// Helper function because serde does not allow literals/constants in #[serde(default = ...)] attributes.
/// See also https://github.com/serde-rs/serde/issues/368
const fn default_limit() -> i32 {
    DEFAULT_ENTRIES_PER_PAGE
}

const fn is_default_entries_per_page(limit: &i32) -> bool {
    *limit == DEFAULT_ENTRIES_PER_PAGE
}

// Helper function needed because serde's flatten attribute does not work with non-self describing data formats (such as url-encoding) - it thinks everything is a string.
// See also https://github.com/nox/serde_urlencoded/issues/33
fn from_str<'de, D, S>(deserializer: D) -> Result<S, D::Error>
where
    D: serde::Deserializer<'de>,
    S: std::str::FromStr,
    S::Err: Display,
{
    let s = <&str as serde::Deserialize>::deserialize(deserializer)?;
    S::from_str(&s).map_err(|err| D::Error::custom(err.to_string()))
}

fn from_str_non_nullable<'de, S, D>(deserializer: D) -> Result<Option<S>, D::Error>
where
    D: serde::Deserializer<'de>,
    S: std::str::FromStr,
    S::Err: Display,
{
    non_nullable::<'de, &'de str, D>(deserializer)?
        .map(|s| S::from_str(&s).map_err(|err| D::Error::custom(err.to_string())))
        .transpose()
}
