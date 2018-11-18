#[macro_use]
pub mod user;
pub mod audit;
pub mod demon;
pub mod player;
pub mod record;
pub mod submitter;

pub use self::{
    demon::Demon,
    player::Player,
    record::Record,
    submitter::Submitter,
    user::{Permissions, User},
};
/*
/// Trait that marks its implementors as models
pub trait Model {
    /// The database column this [`Model`] maps to
    type Columns: Expression;

    /// The database table this [`Model`] maps to
    type Table: SelectDsl<Self::Columns> + QuerySource;

    /// Constructs a simple diesel `SELECT`-query that returns all columns of rows of this
    /// [`Model`]'s from the database
    fn all() -> diesel::dsl::Select<Self::Table, Self::Columns>;
}
*/
