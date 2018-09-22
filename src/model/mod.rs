use crate::schema::*;

pub mod audit;
pub mod demon;
pub mod player;
pub mod record;
pub mod submitter;

pub use self::{demon::Demon, player::Player, record::Record, submitter::Submitter};

#[derive(Queryable, Insertable, Debug)]
#[table_name = "members"]
pub struct User {
    #[column_name = "member_id"]
    id: i32,

    name: String,
    display_name: Option<String>,
    youtube_channel: Option<String>,

    password_hash: Vec<u8>,
    password_salt: Vec<u8>,

    // TODO: deal with this
    permissions: Vec<u8>,
}
