use std::fmt::{Debug, Display};

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
        Self {
            before: None,
            after: None,
            limit: DEFAULT_ENTRIES_PER_PAGE,
        }
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

/// Enum describing what is going on "around" a page returned by [`Pagination::page`].
///
/// Describes whether [`Pagination::Item`] matching all properties of a given [`Pagination`] exist
/// with an id lower/larger than the smallest/largest on a given page
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PageContext {
    /// The page contains all possible items matching the given [`Pagination`] query.
    /// No further pages exist.
    ///
    /// For example, given a list of ids such as `[1, 3, 5]`, a request such as `?after=0&before=6`
    /// would return the page`[1, 3, 5]`, meaning this page is `Standalone`.
    Standalone,

    /// There exist more items matching the given [`Pagination`] query whose ids are less than
    /// the smallest of this page.
    ///
    /// For example, given a list of ids such as `[1, 3, 5]`, a request such as `?after=1&before=6`
    /// would return the page`[3, 5]`, meaning there exists a previous page containing just the item `1`.
    HasPrevious,

    /// There exist more items matching the given [`Pagination`] query whose ids are greater than
    /// the largest of this page.
    ///
    /// For example, given a list of ids such as `[1, 3, 5]`, a request such as `?after=0&before=5`
    /// would return the page`[1, 3]`, meaning there exists a next page containing just the item `5`.
    HasNext,

    /// There exist more items matching the given [`Pagination`] query, some whose ids are less than
    /// the smallest of this page, and some whose ids are greater than the greatest of this page.
    ///
    /// For example, given a list of ids such as `[1, 3, 5]`, a request such as `?after=1&before=5`
    /// would return the page`[3]`, meaning there exists a previous page containing just the item `1`,
    /// and a next page containing just the item `5`.
    HasPreviousAndNext,
}

impl PageContext {
    pub fn has_next(&self) -> bool {
        matches!(self, PageContext::HasNext | PageContext::HasPreviousAndNext)
    }

    pub fn has_previous(&self) -> bool {
        matches!(self, PageContext::HasPrevious | PageContext::HasPreviousAndNext)
    }
}

pub trait PaginationQuery: Serialize + Debug {
    fn parameters(&self) -> PaginationParameters;
    fn with_parameters(&self, parameters: PaginationParameters) -> Self;
}

#[allow(async_fn_in_trait)]
pub trait Paginatable<Q: PaginationQuery>: Serialize + Sized {
    /// Returns a page of objects matching the query described by tthe given [`PaginationQuery`].
    ///
    /// The returned list of objects must have the following properties:
    /// - They are sorted in ascending order according to the value of [`pagination_id`].
    /// - Their ids are consecutive, meaning if the object at index `i` in the list has ID `a`, and
    ///   the object at index `i + 1` has id `b`, then there exists no object also matching all conditions
    ///   of this `Pagination` in the _database_ with an ID `c` such that `a < c < b`.
    /// - If the `after` parameter of the query's associated [`PaginationParameters`] is set, then the first
    ///   object in the returned list must have the smallest ID out of all objects matching the given
    ///   query greater than `after`.
    /// - If the `after` parameter is not set, but `before` is, then the last object in the returned
    ///   list must have the greatest ID out of all objects matching the given query smaller than
    ///   `before`.
    ///
    /// The returned [`PageContext`] should describe whether more pages surrounding this page exist which
    /// match all conditions of this [`Pagination`] object, with the exception of the `before` and `after` fields!
    /// HOWEVER, if both `before` and `after` are set, then it should be [`PageContext::Standalone`].
    ///
    /// The number of items in the returned `Vec` must not exceed [`PaginationParameters::limit`].
    async fn page(query: &Q, connection: &mut PgConnection) -> Result<(Vec<Self>, PageContext), sqlx::Error>;

    async fn first_and_last(connection: &mut PgConnection) -> Result<Option<(i32, i32)>, sqlx::Error>;

    fn pagination_id(&self) -> i32;
}

/// Historically, pointercrate has been determining whether a new page exists by simply incrementing the "limit" parameter
/// by one, and seeing if we can get one extra object from the database. This object was then popped from the results,
/// and its presence indicated that "another page in the same direction" existed - e.g. if `after` was specified, it
/// meant that a "next" should be generated, and if `before` was specified (but not after), it meant that a "prev" should
/// be generated. While this logic is correct, it should never have leaked outside of the `page` implementation and into
/// the actual pagination API.
///
/// Additionally, pointercrate assumes that a previous page exists whenever "after" is set, and that a next page exists
/// whenever "previous" is set (but that we have a standalong page if _both_ are set). This is not sound (we should really be
/// trying to find an "extra" object at the other end of the list), but fixing this would require a bigger refractor than I am
/// willing to do at the time of writing this.
///
/// Lastly, pointercrate used to return the object list in reverse if `before` but not `after` was set, and left it up
/// to the caller to reverse it. That, too, is an implementation detail that should never become API.
///
/// This compat function tries to fix these up as best as it can - it reverses the given list of objects if needed, and translates
/// the "extra" object into a `PageContext`. It doesn't solve the second point though.
#[doc(hidden)]
pub fn __pagination_compat<T>(params: &PaginationParameters, mut objects: Vec<T>) -> (Vec<T>, PageContext) {
    let has_followup_page = objects.len() > params.limit as usize;

    if has_followup_page {
        objects.pop();
    }

    let ctx = match (params.before, params.after) {
        (Some(_), None) => {
            objects.reverse();

            if has_followup_page {
                PageContext::HasPreviousAndNext
            } else {
                PageContext::HasNext
            }
        },
        (None, Some(_)) => {
            if has_followup_page {
                PageContext::HasPreviousAndNext
            } else {
                PageContext::HasPrevious
            }
        },
        (Some(_), Some(_)) => PageContext::Standalone,
        (None, None) => {
            if has_followup_page {
                PageContext::HasNext
            } else {
                PageContext::Standalone
            }
        },
    };

    (objects, ctx)
}

#[macro_export]
macro_rules! first_and_last {
    ($table_name: expr, $id_column: expr) => {
        async fn first_and_last(connection: &mut PgConnection) -> std::result::Result<Option<(i32, i32)>, sqlx::Error> {
            let row = sqlx::query!(
                "SELECT CAST(MIN(" + $id_column + ") AS INTEGER), CAST(MAX(" + $id_column + ") AS INTEGER) FROM " + $table_name
            )
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
