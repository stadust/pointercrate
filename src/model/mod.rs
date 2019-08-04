#[macro_use]
pub mod user;
pub mod demonlist;
pub mod nationality;

pub use self::user::User;

use diesel::{
    dsl::{Eq, Filter, Select},
    expression::{AsExpression, Expression, SelectableExpression},
    pg::Pg,
    query_builder::{BoxedSelectStatement, QueryFragment, SelectStatement},
    query_dsl::{boxed_dsl::BoxedDsl, filter_dsl::FilterDsl, select_dsl::SelectDsl},
    query_source::Column,
    ExpressionMethods, QuerySource,
};

pub type All<M> = Select<SelectStatement<<M as Model>::From>, <M as Model>::Selection>;

pub trait Model {
    type From: QuerySource;
    type Selection: Expression + SelectableExpression<Self::From> + QueryFragment<Pg> + 'static;

    fn from() -> Self::From;

    fn selection() -> Self::Selection;

    fn all() -> Select<SelectStatement<Self::From>, Self::Selection> {
        SelectStatement::simple(Self::from()).select(Self::selection())
    }

    fn boxed_all<'a>(
    ) -> BoxedSelectStatement<'a, <Self::Selection as Expression>::SqlType, Self::From, Pg> {
        BoxedDsl::internal_into_boxed(
            SelectStatement::simple(Self::from()).select(Self::selection()),
        )
    }
}

trait By<T: Column + Default + ExpressionMethods, U: AsExpression<T::SqlType>>: Model {
    fn with(u: U) -> diesel::dsl::Eq<T, U> {
        T::default().eq(u)
    }

    fn by(u: U) -> Filter<All<Self>, Eq<T, U>>
    where
        All<Self>: FilterDsl<Eq<T, U>>,
    {
        Self::all().filter(Self::with(u))
    }
}
