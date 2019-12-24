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

macro_rules! by {
    ($method_name: ident, $column: path, $rust_type: ty) => {
        fn $method_name(
            value: $rust_type,
        ) -> diesel::dsl::Filter<crate::model::All<Self>, diesel::dsl::Eq<$column, $rust_type>> {
            use diesel::{ExpressionMethods, QueryDsl};

            Self::all().filter($column.eq(value))
        }
    };
}

#[macro_use]
pub mod user;
pub mod demonlist;
pub mod nationality;

pub use self::user::User;

use diesel::{
    associations::HasTable,
    dsl::{Find, Select},
    expression::Expression,
    helper_types::IntoBoxed,
    pg::Pg,
    query_dsl::{boxed_dsl::BoxedDsl, filter_dsl::FindDsl, select_dsl::SelectDsl},
    Identifiable,
};

type All<T> = Select<<T as HasTable>::Table, <T as Model>::Selection>;

pub trait Model: HasTable
where
    Self::Table: SelectDsl<Self::Selection>,
{
    type Selection: Expression;

    // Sadly, we cannot solved this via `Self::Selection::default()` because some of our tables
    // (records_pds) use so many columns that the resulting tuple doesn't implement Default anymore
    fn selection() -> Self::Selection;

    fn all() -> All<Self> {
        SelectDsl::select(Self::table(), Self::selection())
    }

    fn boxed_all() -> IntoBoxed<'static, All<Self>, Pg>
    where
        All<Self>: BoxedDsl<'static, Pg>,
    {
        Self::all().internal_into_boxed()
    }

    fn find<'ident>(
        id: <&'ident Self as Identifiable>::Id,
    ) -> Find<All<Self>, <&'ident Self as Identifiable>::Id>
    where
        &'ident Self: Identifiable, // + FindDsl<Self::Id>,
        All<Self>: FindDsl<<&'ident Self as Identifiable>::Id>,
    {
        FindDsl::find(Self::all(), id)
    }
}
