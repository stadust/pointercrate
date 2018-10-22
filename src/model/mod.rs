#[macro_use]
pub mod user;
pub mod audit;
pub mod demon;
pub mod player;
pub mod record;
pub mod submitter;

pub use self::{demon::Demon, player::Player, record::Record, submitter::Submitter, user::User};

use diesel::{query_dsl::methods::SelectDsl, Expression, QuerySource};

pub trait Model {
    type Columns: Expression;
    type Table: SelectDsl<Self::Columns> + QuerySource;

    fn all() -> diesel::dsl::Select<Self::Table, Self::Columns>;


}
