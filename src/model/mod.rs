#[macro_use]
pub mod user;
pub mod creator;
pub mod demon;
pub mod player;
pub mod record;
pub mod submitter;

pub use self::{demon::Demon, player::Player, record::Record, submitter::Submitter, user::User};

use diesel::{
    dsl::Select,
    expression::{Expression, SelectableExpression},
    pg::Pg,
    query_builder::{BoxedSelectStatement, QueryFragment, SelectStatement},
    query_dsl::{boxed_dsl::BoxedDsl, select_dsl::SelectDsl},
    QuerySource,
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
