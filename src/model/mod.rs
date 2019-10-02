//! Module containing all the structs modelling objects in the database
//!
//! For every object, there are multiple representations:
//! * A direct mapping to the underlying database table. This struct is always prefixed with
//!   `Database`
//! * A struct modelling the object with nearly all additional information available. This
//!   representation is used for the endpoints that return a single object. This struct is prefixed
//!   with `Full`. This is generally also the struct with a `Patch` implementation.
//! * A struct modelling the object the way it should be presented by the paginating endpoints. This
//!   struct doesn't have any special prefix.
//! * A variety of "minimal" representations. These are returned if the object is presented as
//!   another object's field. They are always prefixed with `Minimal`. Sometimes, more than one
//!   minimal representation exists because different enclosing objects render different parts of
//!   the object obsolete. In these cases a short suffix shows which parts of the object is
//!   modelled.
//! Only the `Database` representation always exists. The others are occasionally not necessary
//! distinct from each other

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
