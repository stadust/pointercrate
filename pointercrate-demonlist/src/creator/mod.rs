// pub use self::post::PostCreator;
pub use self::get::{created_by, creators_of};
use derive_more::Display;
pub use post::PostCreator;

mod delete;
mod get;
mod post;

#[derive(Debug, Display, Hash)]
#[display(fmt = "creator with id {} on demon {}", creator, demon)]
pub struct Creator {
    demon: i32,
    creator: i32,
}
