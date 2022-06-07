pub use paginate::{ListedClaim, PlayerClaimPagination};
pub use patch::PatchPlayerClaim;
use serde::{Deserialize, Serialize};

mod delete;
mod get;
mod paginate;
mod patch;
mod put;

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct PlayerClaim {
    pub user_id: i32,
    pub player_id: i32,
    pub verified: bool,

    /// Whether the pointercrate user claiming this player has requested submissions to be locked,
    /// meaning records for this player can only be submitted while the claimer is logged in.
    pub lock_submissions: bool,
}
