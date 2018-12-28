use crate::Result;
use diesel::{
    expression::Expression,
    pg::{Pg, PgConnection},
    query_builder::BoxedSelectStatement,
};
use serde::{Deserialize, Serialize};

pub trait Paginator: Sized + Serialize
where
    for<'de> Self: Deserialize<'de>,
{
    type Selection: Expression;
    type QuerySource;

    fn base<'a>(
    ) -> BoxedSelectStatement<'a, <Self::Selection as Expression>::SqlType, Self::QuerySource, Pg>;

    fn next(&self, connection: &PgConnection) -> Result<Option<Self>>;
    fn prev(&self, connection: &PgConnection) -> Result<Option<Self>>;
    fn first(&self, connection: &PgConnection) -> Result<Option<Self>>;
    fn last(&self, connection: &PgConnection) -> Result<Option<Self>>;
}

pub trait Paginate<P: Paginator>: Sized {
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
        fn filter<'a, ST>(&'a self, mut query: BoxedSelectStatement<'a, ST, <Self as Paginator>::QuerySource, Pg>) -> BoxedSelectStatement<'a, ST, <Self as Paginator>::QuerySource, Pg>
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

macro_rules! navigation {
    // TODO: maybe do the same with limit just for completeness sake
    ($table: ident, $column: ident) => {
        navigation!($table, $column, i32, before, after);
    };

    ($table: ident, $column: ident, $before: ident, $after: ident) => {
        navigation!($table, $column, i32, $before, $after);
    };

    ($table: ident, $column: ident, $column_type: ty, $before: ident, $after: ident) => {
        fn next(&self, connection: &PgConnection) -> Result<Option<Self>> {
            use diesel::{ExpressionMethods, QueryDsl, select, dsl::exists, RunQueryDsl, OptionalExtension};

            let after = if let Some(id) = self.$before {
                if select(exists(self.filter(Self::base().filter($table::$column.ge(id))))).get_result(connection)? {
                    id - 1
                } else {
                    return Ok(None)
                }
            }else {
                let limit = self.limit.unwrap_or(50);

                let mut base = self.filter(Self::base().select($table::$column));

                if let Some(after) = self.$after {
                    base = base.filter($table::$column.gt(after));
                }

                let after = base
                    .offset(limit)
                    .limit(limit + 1)
                    .get_result(connection)
                    .map(|id: $column_type| id - 1)
                    .optional()?;

                match after {
                    Some(id) => id,
                    None => return Ok(None)
                }
            };

            Ok(Some(Self {
                $after: Some(after),
                $before:None,
                ..self.clone()
            }))
        }

        fn prev(&self, connection: &PgConnection) -> Result<Option<Self>> {
            use diesel::{ExpressionMethods, QueryDsl, select, dsl::exists, RunQueryDsl, OptionalExtension};

            let before = if let Some(id) = self.$after {
                if select(exists(self.filter(Self::base().filter($table::$column.le(id))))).get_result(connection)? {
                    id + 1
                } else {
                    return Ok(None)
                }
            }else {
                let limit = self.limit.unwrap_or(50);

                let mut base = self.filter(Self::base().select($table::$column));

                if let Some(before) = self.$before {
                    base = base.filter($table::$column.lt(before));
                }

                let before = base
                    .order_by($table::$column.desc())
                    .offset(limit)
                    .limit(limit + 1)
                    .get_result(connection)
                    .map(|id: $column_type| id + 1)
                    .optional()?;

                match before {
                    Some(id) => id,
                    None => return Ok(None)
                }
            };

            Ok(Some(Self {
                $before: Some(before),
                $after: None,
                ..self.clone()
            }))
        }

        fn first(&self, connection: &PgConnection) -> Result<Option<Self>> {
            use diesel::{dsl::min, QueryDsl};

            Ok(
                self.filter(Self::base().select(min($table::$column)))
                .get_result::<Option<$column_type>>(connection)?
                .map(|id: $column_type| Self{$after: Some(id - 1), $before:None,..self.clone()})
            )
        }

        fn last(&self, connection: &PgConnection) -> Result<Option<Self>> {
            use diesel::{dsl::max, QueryDsl};

            Ok(
                self.filter(Self::base().select(max($table::$column)))
                    .get_result::<Option<$column_type>>(connection)?
                    .map(|id: $column_type| Self{$before: Some(id + 1), $after:None, ..self.clone()})
            )
        }
    };
}
