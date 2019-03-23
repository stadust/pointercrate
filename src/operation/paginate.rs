use crate::{model::Model, Result};
use diesel::{
    dsl::{exists, max, min},
    expression::{AsExpression, NonAggregate},
    pg::{Pg, PgConnection},
    query_builder::{BoxedSelectStatement, QueryFragment},
    select,
    sql_types::{HasSqlType, NotNull, SqlOrd},
    AppearsOnTable, Expression, ExpressionMethods, OptionalExtension, QueryDsl, QuerySource,
    Queryable, RunQueryDsl, SelectableExpression,
};
use serde::{Deserialize, Serialize};
use crate::context::RequestContext;

pub trait Paginator: Sized + Serialize
where
    for<'de> Self: Deserialize<'de>,
    <Self::PaginationColumn as Expression>::SqlType: NotNull + SqlOrd,
    <<Self::Model as Model>::From as QuerySource>::FromClause: QueryFragment<Pg>,
    Pg: HasSqlType<<Self::PaginationColumn as Expression>::SqlType>,
    <Self::PaginationColumnType as AsExpression<
        <Self::PaginationColumn as Expression>::SqlType,
    >>::Expression: AppearsOnTable<
        <Self::Model as Model>::From,
    >,
    <Self::PaginationColumnType as AsExpression<<Self::PaginationColumn as Expression>::SqlType>>::Expression: NonAggregate,
    <Self::PaginationColumnType as AsExpression<<Self::PaginationColumn as Expression>::SqlType>>::Expression: QueryFragment<Pg>
{
    type Model: Model;
    // Columns are effectively unit structs and diesel always derives Default for them
    type PaginationColumn: Default
        + SelectableExpression<<Self::Model as Model>::From>
        + QueryFragment<Pg>
        + ExpressionMethods + NonAggregate;
    type PaginationColumnType: Queryable<<Self::PaginationColumn as Expression>::SqlType, Pg>
        + diesel::expression::AsExpression<
            <Self::PaginationColumn as Expression>::SqlType,
        >
        + Clone;

    fn filter<'a, ST>(
        &'a self,
        query: BoxedSelectStatement<
            'a,
            ST,
            <<Self as Paginator>::Model as crate::model::Model>::From,
            Pg,
        >,
    ) -> BoxedSelectStatement<'a, ST, <<Self as Paginator>::Model as crate::model::Model>::From, Pg>;

    fn page(
        &self, last_on_page: Option<Self::PaginationColumnType>,
        first_on_page: Option<Self::PaginationColumnType>,
    ) -> Self;

    fn limit(&self) -> i64;
    fn before(&self) -> Option<Self::PaginationColumnType>;
    fn after(&self) -> Option<Self::PaginationColumnType>;

    fn first(&self, connection: &PgConnection) -> Result<Option<Self>> {
        Ok(self
            .filter(Self::Model::boxed_all().select(min(Self::PaginationColumn::default())))
            .get_result::<Option<Self::PaginationColumnType>>(connection)?
            .map(|id| self.page(None, Some(id))))
    }

    fn last(&self, connection: &PgConnection) -> Result<Option<Self>> {
        Ok(self
            .filter(Self::Model::boxed_all().select(max(Self::PaginationColumn::default())))
            .get_result::<Option<Self::PaginationColumnType>>(connection)?
            .map(|id| self.page(Some(id), None)))
    }

    /// Returns the first id that could theoretically be on the next page, if the associated object exists
    fn next(&self, connection: &PgConnection) -> Result<Option<Self>> {
        // If the current request had a 'before' value set, we check if object with ids greater than or equal to that value exist.
        // If they, do, the first id that could be on the next page is simply our 'before' value

        let after = if let Some(id) = self.before() {
            if select(exists(self.filter(Self::Model::boxed_all().filter(Self::PaginationColumn::default().ge(id.clone()))))).get_result(connection)? {
                Some(id)
            } else {
                None
            }
        } else {
            // Otherwise, we simply apply the query that returns the current page, offset by limit and then get the next element.
            // If it exists, its the first element on the next page. If it doesn't, there is no next page

            let limit = self.limit();

            let mut base = self.filter(Self::Model::boxed_all().select(Self::PaginationColumn::default()));

            if let Some(after) = self.after() {
                base = base.filter(Self::PaginationColumn::default().gt(after));
            }

            base
                .order_by(Self::PaginationColumn::default())
                .offset(limit)
                .limit(1)
                .get_result(connection)
                .optional()?
        };

        Ok(after.map(|value| self.page(None, Some(value))))
    }

    /// Returns the last id that could theoretically be on the previous page, if the associated object exists
    fn prev(&self, connection: &PgConnection) -> Result<Option<Self>> {
        // If the current request had an 'after' value set, we check if object with ids lesser than or equal to that value exist.
        // If they, do, the last id that could be on the previous page is simply our 'after' value
        let before = if let Some(id) = self.after() {
            if select(exists(self.filter(
                Self::Model::boxed_all().filter(Self::PaginationColumn::default().le(id.clone())),
            )))
            .get_result(connection)?
            {
                Some(id)
            } else {
                None
            }
        } else if self.before().is_some() {
            // Otherwise, we simply apply the query that returns the current page, reverse the order, offset by limit and then get the next element.
            // If it exists, its the last element on the previous page. If it doesn't, there is no previous page
            let limit = self.limit();

            let mut base =
                self.filter(Self::Model::boxed_all().select(Self::PaginationColumn::default()));

            if let Some(before) = self.before() {
                base = base.filter(Self::PaginationColumn::default().lt(before));
            }

            base
                .order_by(Self::PaginationColumn::default().desc())
                .offset(limit)
                .limit(1)
                .get_result(connection)
                .optional()?
        } else {
            // If there are no 'after' and 'before' values set, we know we are on the first page. There is no previous page to the first page
            None
        };

        Ok(before.map(|value| self.page(Some(value), None)))
    }
}

pub trait Paginate<P: Paginator<Model = Self>>: Model + Sized
where
    <P::PaginationColumn as Expression>::SqlType: NotNull + SqlOrd,
    <<P::Model as Model>::From as QuerySource>::FromClause: QueryFragment<Pg>,
    Pg: HasSqlType<<P::PaginationColumn as Expression>::SqlType>,
    P::PaginationColumn:
        Default + SelectableExpression<<P::Model as Model>::From> + QueryFragment<Pg>,
    P::PaginationColumn: SelectableExpression<<P::Model as Model>::From>,
    <P::PaginationColumnType as AsExpression<
        <P::PaginationColumn as Expression>::SqlType,
    >>::Expression: AppearsOnTable<<P::Model as Model>::From>,
    <P::PaginationColumnType as AsExpression<
        <P::PaginationColumn as Expression>::SqlType,
    >>::Expression: NonAggregate,
    <P::PaginationColumnType as AsExpression<
        <P::PaginationColumn as Expression>::SqlType,
    >>::Expression: QueryFragment<Pg>,
{
    fn load(paginator: &P, ctx: RequestContext, connection: &PgConnection) -> Result<Vec<Self>>;
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
    ($query: ident[$($table: ident :: $column: ident $op: tt $value: expr),+]) => {
        use diesel::ExpressionMethods;

        $(
            if let Some(ref value) = $value {
                $query = $query.filter(__op!($table :: $column $op value))
            }
        )+
    };
}

macro_rules! filter_method {
    ($table: ident[$($column: ident $op: tt $value: ident),+]) => {
        fn filter<'a, ST>(&'a self, mut query: BoxedSelectStatement<'a, ST, <<Self as Paginator>::Model as crate::model::Model>::From, Pg>) -> BoxedSelectStatement<'a, ST, <<Self as Paginator>::Model as crate::model::Model>::From, Pg>
        {
            filter!(query[
                $(
                    $table::$column $op self.$value
                ),+
            ]);

            query
        }
    };
}

macro_rules! pagination_result {
    ($query: expr, $pagination_data: expr, $before_column: ident, $after_column: ident, $db_column: path, $connection: expr) => {
        if $pagination_data.$after_column.is_none() && $pagination_data.$before_column.is_some() {
            let mut members = $query
                .order_by($db_column.desc())
                .limit($pagination_data.limit())
                .load($connection)?;

            members.reverse();

            Ok(members)
        } else {
            $query
                .order_by($db_column)
                .limit($pagination_data.limit())
                .load($connection)
                .map_err(PointercrateError::database)
        }
    };
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
