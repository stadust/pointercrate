use pointercrate_core::permission::{Permission, PermissionsManager};
use pointercrate_user::ADMINISTRATOR;

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

pub const LIST_HELPER: Permission = Permission::new("List Helper", 0x2);
pub const LIST_MODERATOR: Permission = Permission::new("List Moderator", 0x4);
pub const LIST_ADMINISTRATOR: Permission = Permission::new("List Administrator", 0x8);

pub fn default_permissions_manager() -> PermissionsManager {
    PermissionsManager::new(vec![ADMINISTRATOR, LIST_HELPER, LIST_MODERATOR, LIST_ADMINISTRATOR])
        .assigns(ADMINISTRATOR, LIST_ADMINISTRATOR)
        .assigns(ADMINISTRATOR, LIST_MODERATOR)
        .assigns(ADMINISTRATOR, LIST_HELPER)
        .assigns(LIST_ADMINISTRATOR, LIST_MODERATOR)
        .assigns(LIST_ADMINISTRATOR, LIST_HELPER)
        .implies(LIST_ADMINISTRATOR, LIST_MODERATOR)
        .implies(LIST_MODERATOR, LIST_HELPER)
}
