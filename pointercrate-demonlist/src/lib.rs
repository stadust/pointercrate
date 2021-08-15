use pointercrate_core::permission::Permission;

#[macro_use]
pub mod demon;
pub mod config;
pub mod creator;
pub mod error;
pub mod nationality;
pub mod player;
pub mod record;
pub mod submitter;
mod video;

pub const LIST_HELPER: Permission = Permission::new(String::from("List Helper"), 0x2);
pub const LIST_MODERATOR: Permission = Permission::new(String::from("List Moderator"), 0x4);
pub const LIST_ADMINISTRATOR: Permission = Permission::new(String::from("List Administrator"), 0x8);
