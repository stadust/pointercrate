use crate::Result;
use diesel::pg::PgConnection;

pub trait Paginator: Sized {
    fn next(&self, connection: &PgConnection) -> Result<Option<Self>>;
    fn prev(&self, connection: &PgConnection) -> Result<Option<Self>>;
    fn first(&self, connection: &PgConnection) -> Result<Option<Self>>;
    fn last(&self, connection: &PgConnection) -> Result<Option<Self>>;
}

pub trait Paginate<P: Paginator>: Sized {
    fn load(&self, paginator: P, connection: &PgConnection) -> Result<Vec<Self>>;
}

macro_rules! __op {
    ($table: ident :: $column: ident = $value: expr) => {
        $table::$column.eq($value)
    };
    ($table: ident :: $column: ident < $value: expr) => {
        members::$column.lt($value)
    };
    ($table: ident :: $column: ident > $value: expr) => {
        members::$column.gt($value)
    };
    ($table: ident :: $column: ident <= $value: expr) => {
        members::$column.le($value)
    };
    ($table: ident :: $column: ident >= $value: expr) => {
        members::$column.ge($value)
    };
}

macro_rules! filter {
    ($query: ident[$($table: ident :: $column: ident $op: tt $value: expr),+]) => {
        $(
            if let Some(ref value) = $value {
                $query = $query.filter(__op!($table :: $column $op value))
            }
        )+
    };
}

macro_rules! filter_method {
    ($table: ident[$($column: ident $op: tt $value: ident),+]) => {
        fn filter<'a, ST>(&'a self, mut query: BoxedSelectStatement<'a, ST, $table::table, Pg>) -> BoxedSelectStatement<'a, ST, $table::table, Pg> {
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
            let after = if let Some(id) = self.$before {
                if select(exists(self.filter($table::table.filter($table::$column.ge(id)).into_boxed()))).get_result(connection)? {
                    id - 1
                } else {
                    return Ok(None)
                }
            }else {
                let limit = self.limit.unwrap_or(50);

                let mut base = self.filter($table::table.select($table::$column).into_boxed());

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
            let before = if let Some(id) = self.$after {
                if select(exists(self.filter($table::table.filter($table::$column.le(id)).into_boxed()))).get_result(connection)? {
                    id + 1
                } else {
                    return Ok(None)
                }
            }else {
                let limit = self.limit.unwrap_or(50);

                let mut base = self.filter($table::table.select($table::$column).into_boxed());

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
            Ok(self.filter($table::table.select(min($table::$column)).into_boxed()).get_result::<Option<$column_type>>(connection)?.map(|id: $column_type| Self{$after: Some(id - 1), $before:None,..self.clone()}))
        }

        fn last(&self, connection: &PgConnection) -> Result<Option<Self>> {
            Ok(self.filter($table::table.select(max($table::$column)).into_boxed()).get_result::<Option<$column_type>>(connection)?.map(|id: $column_type| Self{$before: Some(id + 1), $after:None, ..self.clone()}))
        }
    };
}
