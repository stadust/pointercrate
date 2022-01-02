pub use paginate::{ListedClaim, PlayerClaimPagination};
pub use patch::PatchVerified;
use serde::Serialize;

mod delete;
mod get;
mod paginate;
mod patch;
mod put;

#[derive(Serialize, Debug)]
pub struct PlayerClaim {
    pub user_id: i32,
    pub player_id: i32,
    pub verified: bool,
}
