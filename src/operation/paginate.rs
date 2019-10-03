use crate::{context::RequestContext, model::Model, Result};
use diesel::{
    dsl::{max, min},
    expression::{AsExpression, NonAggregate},
    pg::Pg,
    query_builder::{BoxedSelectStatement, QueryFragment},
    sql_types::{HasSqlType, NotNull, SqlOrd},
    AppearsOnTable, Column, Expression, ExpressionMethods, OptionalExtension, QueryDsl,
    QuerySource, Queryable, RunQueryDsl, SelectableExpression, Table,
};
use serde::{Deserialize, Serialize};

pub trait Paginator: Serialize + Sized
where
    for<'de> Self: Deserialize<'de>,
{
    type On: Ord;

    fn limit(&self) -> u8;
    fn after(&self) -> Option<&Self::On>;
    fn before(&self) -> Option<&Self::On>;

    fn first(&self, ctx: RequestContext) -> Result<Option<Self>>;
    fn last(&self, ctx: RequestContext) -> Result<Option<Self>>;
    fn previous(&self, ctx: RequestContext) -> Result<Option<Self>>;
    fn next(&self, ctx: RequestContext) -> Result<Option<Self>>;
}

pub type PaginatorQuery<'a, T> =
    BoxedSelectStatement<'a, <<T as Table>::AllColumns as Expression>::SqlType, T, Pg>;

// Seriously, fuck diesel
pub trait TablePaginator: Serialize
where
    <Self::PaginationColumn as Expression>::SqlType: NotNull + SqlOrd,
    Pg: HasSqlType<<Self::PaginationColumn as Expression>::SqlType>,
    <Self::Table as QuerySource>::FromClause: QueryFragment<Pg>,
    <Self::ColumnType as AsExpression<<Self::PaginationColumn as Expression>::SqlType>>::Expression:
        AppearsOnTable<Self::Table>,
    <Self::ColumnType as AsExpression<<Self::PaginationColumn as Expression>::SqlType>>::Expression:
        NonAggregate,
    <Self::ColumnType as AsExpression<<Self::PaginationColumn as Expression>::SqlType>>::Expression:
        QueryFragment<Pg>,
{
    type Table: Table;
    type PaginationColumn: Column<Table = Self::Table>
        + Default
        + SelectableExpression<Self::Table>
        + QueryFragment<Pg>
        + ExpressionMethods
        + NonAggregate;
    type ColumnType: Queryable<<Self::PaginationColumn as Expression>::SqlType, Pg>
        + AsExpression<<Self::PaginationColumn as Expression>::SqlType>
        + Ord;

    fn query(&self, ctx: RequestContext) -> PaginatorQuery<Self::Table>;

    fn first_id(&self, ctx: RequestContext) -> Result<Option<Self::ColumnType>> {
        let connection = ctx.connection();

        let query = self.query(ctx);
        let query = query.select(min(Self::PaginationColumn::default()));
        let minimum = query.get_result::<Option<Self::ColumnType>>(connection)?;

        Ok(minimum)
    }

    fn last_id(&self, ctx: RequestContext) -> Result<Option<Self::ColumnType>> {
        let connection = ctx.connection();

        let query = self.query(ctx);
        let query = query.select(max(Self::PaginationColumn::default()));
        let maximum = query.get_result::<Option<Self::ColumnType>>(connection)?;

        Ok(maximum)
    }

    /// Returns the id of the **last object on the current page**. This will be the value required
    /// for the `after` parameter to retrieve the *next* page.
    ///
    /// This does not check if there actually are objects after the last one on the current page. To
    /// check this, compare the returned id with the one returned by [`TablePaginator::last`]
    fn last_id_on_page(
        &self,
        before: Option<Self::ColumnType>,
        after: Option<Self::ColumnType>,
        limit: u8,
        ctx: RequestContext,
    ) -> Result<Option<Self::ColumnType>> {
        let connection = ctx.connection();
        let query = self.query(ctx);

        match before {
            // If the current request had a 'before' value set, we need to get the maximal id
            // smaller than the 'before' value
            Some(id) =>
                query
                    .filter(Self::PaginationColumn::default().lt(id))
                    .select(max(Self::PaginationColumn::default()))
                    .get_result::<Option<Self::ColumnType>>(connection)
                    .map_err(Into::into),
            None => {
                // Otherwise, we simply apply the query that returns the current page, order by id
                // in descending order, and get the first element of the result set
                let mut query = query.select(Self::PaginationColumn::default());

                if let Some(after) = after {
                    query = query.filter(Self::PaginationColumn::default().gt(after));
                }

                query
                    .order_by(Self::PaginationColumn::default().desc())
                    .first(connection)
                    .optional()
                    .map_err(Into::into)
            },
        }
    }

    /// Returns the id of the first object on the *previous* page, if a next page exists.
    ///
    /// If `before` is set to the returned value plus one, the previous page can be retrieved
    fn last_id_previous_page(
        &self,
        before: Option<Self::ColumnType>,
        after: Option<Self::ColumnType>,
        limit: u8,
        ctx: RequestContext,
    ) -> Result<Option<Self::ColumnType>> {
        let connection = ctx.connection();

        match after {
            // If we have an after value, we just need to get the maximal id smaller than or equal
            // to it
            Some(id) =>
                self.query(ctx)
                    .filter(Self::PaginationColumn::default().le(id))
                    .select(max(Self::PaginationColumn::default()))
                    .get_result::<Option<Self::ColumnType>>(connection)
                    .map_err(Into::into),
            None => {
                match before {
                    // In this case we need to reverse the ordering of the IDs before our `before`
                    // value, offset by limit and get the first id
                    Some(id) =>
                        self.query(ctx)
                            .filter(Self::PaginationColumn::default().lt(id))
                            .select(Self::PaginationColumn::default())
                            .order_by(Self::PaginationColumn::default().desc())
                            .offset(limit as i64)
                            .first(connection)
                            .optional()
                            .map_err(Into::into),
                    // If no before and no after value is set, we are on the first page. No previous
                    // page exists by definition then
                    None => Ok(None),
                }
            },
        }
    }

    /// Returns the id of the first object on the *next* page, if a next page exists
    fn first_id_next_page(
        &self,
        before: Option<Self::ColumnType>,
        after: Option<Self::ColumnType>,
        limit: u8,
        ctx: RequestContext,
    ) -> Result<Option<Self::ColumnType>> {
        let connection = ctx.connection();

        match before {
            // If we have a before value, we need to get the minimal ID greater than or equal to it
            Some(id) =>
                self.query(ctx)
                    .filter(Self::PaginationColumn::default().ge(id))
                    .select(min(Self::PaginationColumn::default()))
                    .get_result::<Option<Self::ColumnType>>(connection)
                    .map_err(Into::into),
            // if we do not have a before value we can simply offset by limit and get the first ID
            // (if that exists)
            None => {
                let mut query = if let Some(after) = after {
                    self.query(ctx)
                        .filter(Self::PaginationColumn::default().gt(after))
                } else {
                    self.query(ctx)
                };

                query
                    .offset(limit as i64)
                    .order_by(Self::PaginationColumn::default())
                    .select(Self::PaginationColumn::default())
                    .first(connection)
                    .optional()
                    .map_err(Into::into)
            },
        }
    }
}

macro_rules! delegate_to_table_paginator {
    ($paginator: ty) => {
        delegate_to_table_paginator!($paginator, limit, before_id, after_id);
    };

    ($paginator: ty, $limit: ident, $before: ident, $after: ident) => {
        impl Paginator for $paginator {
            type On = <$paginator as TablePaginator>::ColumnType;

            fn limit(&self) -> u8 {
                self.$limit.unwrap_or(50u8)
            }

            fn before(&self) -> Option<&Self::On> {
                self.$before.as_ref()
            }

            fn after(&self) -> Option<&Self::On> {
                self.$after.as_ref()
            }

            fn first(&self, _: RequestContext) -> Result<Option<Self>> {
                Ok(Some(Self {
                    $before: None,
                    $after: None,
                    ..self.clone()
                }))
            }

            fn last(&self, ctx: RequestContext) -> Result<Option<Self>> {
                Ok(self.last_id(ctx)?.map(|last| {
                    Self {
                        $before: Some(last + 1),
                        $after: None,
                        ..self.clone()
                    }
                }))
            }

            fn next(&self, ctx: RequestContext) -> Result<Option<Self>> {
                let first_on_next_page =
                    self.first_id_next_page(self.$before, self.$after, self.limit(), ctx)?;

                log::debug!(
                    "Pagination for {}: First on next page is {:?}",
                    stringify!($paginator),
                    first_on_next_page
                );

                Ok(first_on_next_page.map(|id| {
                    Self {
                        $before: None,
                        $after: Some(id - 1),
                        ..self.clone()
                    }
                }))
            }

            fn previous(&self, ctx: RequestContext) -> Result<Option<Self>> {
                let last_on_previous_page =
                    self.last_id_previous_page(self.$before, self.$after, self.limit(), ctx)?;

                log::debug!(
                    "Pagination for {}: Last on previous page is {:?}",
                    stringify!($paginator),
                    last_on_previous_page
                );

                Ok(last_on_previous_page.map(|id| {
                    Self {
                        $before: Some(id + 1),
                        $after: None,
                        ..self.clone()
                    }
                }))
            }
        }
    };
}

pub trait Paginate<P: Paginator>: Model + Sized {
    fn load(paginator: &P, ctx: RequestContext) -> Result<Vec<Self>>;
}

macro_rules! __op {
    ($table: ident :: $column: ident = $value: expr) => {
        $table::$column.eq($value)
    };
    ($table: ident :: $column: ident < $value: expr) => {
        $table::$column.lt($value)
    };
    ($table: ident :: $column: ident > $value: expr) => {
        $table::$column.gt($value)
    };
    ($table: ident :: $column: ident <= $value: expr) => {
        $table::$column.le($value)
    };
    ($table: ident :: $column: ident >= $value: expr) => {
        $table::$column.ge($value)
    };
}

macro_rules! filter {
    ($query: ident[$($table: ident :: $column: ident $op: tt $value: expr),+]) => {{
        use diesel::ExpressionMethods;

        $(
            if let Some(ref value) = $value {
                $query = $query.filter(__op!($table :: $column $op value))
            }
        )+
    }};
}

macro_rules! pagination_result {
    ($query: expr, $pagination_data: expr, $before_column: ident, $after_column: ident, $db_column: path, $connection: expr) => {{
        use crate::error::PointercrateError;
        use diesel::{ExpressionMethods, RunQueryDsl};
        if $pagination_data.$after_column.is_none() && $pagination_data.$before_column.is_some() {
            let mut members = $query
                .order_by($db_column.desc())
                .limit($pagination_data.limit() as i64)
                .load($connection)?;

            members.reverse();

            Ok(members)
        } else {
            $query
                .order_by($db_column)
                .limit($pagination_data.limit() as i64)
                .load($connection)
                .map_err(PointercrateError::database)
        }
    }};

    ($query: expr, $pagination_data: expr, $db_column: path, $connection: expr) => {
        pagination_result!(
            $query,
            $pagination_data,
            before_id,
            after_id,
            $db_column,
            $connection
        )
    };
}
