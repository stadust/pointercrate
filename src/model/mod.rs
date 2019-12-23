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
    expression::{AsExpression, Expression},
    helper_types::IntoBoxed,
    pg::Pg,
    query_dsl::{boxed_dsl::BoxedDsl, filter_dsl::FilterDsl, select_dsl::SelectDsl},
    ExpressionMethods,
};

pub type All<M> = Select<<M as Model>::From, <M as Model>::Selection>;

pub trait Model {
    type From: SelectDsl<Self::Selection>;
    type Selection: Expression;

    fn from() -> Self::From;

    fn selection() -> Self::Selection;

    fn all() -> Select<Self::From, Self::Selection> {
        SelectDsl::select(Self::from(), Self::selection())
    }

    fn boxed_all() -> IntoBoxed<'static, All<Self>, Pg>
    where
        All<Self>: BoxedDsl<'static, Pg>,
    {
        Self::all().internal_into_boxed()
    }
}

trait By<T, U>: Model
where
    T: Default + ExpressionMethods,
    U: AsExpression<T::SqlType>,
{
    fn by(u: U) -> Filter<All<Self>, Eq<T, U>>
    where
        All<Self>: FilterDsl<Eq<T, U>>,
    {
        Self::all().filter(T::default().eq(u))
    }
}
