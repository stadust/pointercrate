use crate::error::PointercrateError;
use diesel::{
    pg::Pg,
    query_builder::{AsQuery, BoxedSelectStatement},
    PgConnection, Table,
};

pub trait Paginatable: Clone {
    type Table: Table;

    fn filter<'a, ST>(
        &'a self, query: BoxedSelectStatement<'a, ST, Self::Table, Pg>,
    ) -> BoxedSelectStatement<'a, ST, Self::Table, Pg>;

    fn query<'a>(
        &'a self,
    ) -> BoxedSelectStatement<'a, <Self::Table as AsQuery>::SqlType, Self::Table, Pg>;

    /// Gets the `after` value for the query in the `next` link
    ///
    /// + If a `before` value is currently set, `after` will be `Some(before - 1)` if there exists
    /// any object with `id >= before`, or `None` otherwise
    /// + Otherwise, we try to get `limit.unwrap_or(50) + 1` objects and either return the ID of
    /// the (limits + 1)th object - 1, or `None` if the object doesn't exist
    fn next_after(&self, conn: &PgConnection) -> Result<Option<i32>, PointercrateError>;

    /// Gets the `before` value for the query in the `prev` link
    ///
    /// + If a `after` value is currently set, `before` will be `Some(after + 1)` if there exists
    /// any object with `id <= after` or `None` otherwise
    /// + Otherwise, we try to get `limit.unwrap_or(50) + 1` objects in reversed order and
    /// either return the (limits + 1)th object + 1, or `None` if the object doesn't exist
    fn prev_before(&self, conn: &PgConnection) -> Result<Option<i32>, PointercrateError>;

    fn first(&self, conn: &PgConnection) -> Result<Option<i32>, PointercrateError>;
    fn last(&self, conn: &PgConnection) -> Result<Option<i32>, PointercrateError>;

    fn clone_with(&self, after: Option<i32>, before: Option<i32>) -> Self;
}
