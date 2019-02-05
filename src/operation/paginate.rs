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

// TODO: figure out a way to omit some fields if the user requesting the data is missing some
// permissions
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

    fn next(&self, connection: &PgConnection) -> Result<Option<Self>> {
        let after = if let Some(id) = self.before() {
            if select(exists(self.filter(Self::Model::boxed_all().filter(Self::PaginationColumn::default().ge(id.clone()))))).get_result(connection)? {
                id
            } else {
                return Ok(None)
            }
        }else {
            let limit = self.limit();

            let mut base = self.filter(Self::Model::boxed_all().select(Self::PaginationColumn::default()));

            if let Some(after) = self.after() {
                base = base.filter(Self::PaginationColumn::default().gt(after));
            }

            let after = base
                .offset(limit)
                .limit(limit + 1)
                .get_result(connection)
                .optional()?;

            match after {
                Some(id) => id,
                None => return Ok(None)
            }
        };

        Ok(Some(self.page(None, Some(after))))
    }

    fn prev(&self, connection: &PgConnection) -> Result<Option<Self>> {
        let before = if let Some(id) = self.after() {
            if select(exists(self.filter(
                Self::Model::boxed_all().filter(Self::PaginationColumn::default().le(id.clone())),
            )))
            .get_result(connection)?
            {
                id
            } else {
                return Ok(None)
            }
        } else {
            let limit = self.limit();

            let mut base =
                self.filter(Self::Model::boxed_all().select(Self::PaginationColumn::default()));

            if let Some(before) = self.before() {
                base = base.filter(Self::PaginationColumn::default().lt(before));
            }

            let before = base
                .order_by(Self::PaginationColumn::default().desc())
                .offset(limit)
                .limit(limit + 1)
                .get_result(connection)
                .optional()?;

            match before {
                Some(id) => id,
                None => return Ok(None),
            }
        };

        Ok(Some(self.page(Some(before), None)))
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
    fn load(paginator: &P, connection: &PgConnection) -> Result<Vec<Self>>;
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
