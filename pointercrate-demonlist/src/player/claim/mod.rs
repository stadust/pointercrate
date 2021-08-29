pub use paginate::{ListedClaim, PlayerClaimPagination};
pub use patch::PatchVerified;
use serde::Serialize;

mod get;
mod paginate;
mod patch;
mod put;

#[derive(Serialize, Debug)]
pub struct PlayerClaim {
    user_id: i32,
    pub player_id: i32,
    pub verified: bool,
}
