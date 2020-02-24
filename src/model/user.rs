//! Module for general administrative user actions. See [`auth`] for account related user actions
//!
//! Includes:
//! * Deleting other accounts
//! * Modifying other people's accounts (assign permissions, change offensive names, etc)
//! * Querying account information

use crate::{
    error::PointercrateError,
    permissions::{Permissions, PermissionsSet},
    Result,
};
pub use auth::{AuthenticatedUser, Authorization};
use jsonwebtoken::{DecodingKey, EncodingKey};
use log::{debug, warn};
use serde::{Deserialize, Serialize, Serializer};
use std::{
    fmt::{Display, Formatter},
    hash::{Hash, Hasher},
};

mod auth;
mod delete;
mod get;
mod patch;
/*mod delete;
mod paginate;
mod patch;
mod post;*/

// TODO: impl the nationality stuff already in the database
/// Model representing a user in the database
#[derive(Debug, Serialize, Hash, Eq)]
pub struct User {
    /// The [`User`]'s unique ID. This is used to identify users and cannot be changed.
    pub id: i32,

    /// The [`User`]'s unique username. This is used to log-in and cannot be changed.
    pub name: String,

    pub permissions: Permissions,

    /// A user-customizable name for each [`User`].
    ///
    /// If set to anything other than [`None`], the value set here will be displayed everywhere the
    /// username would be displayed otherwise. This value is not guaranteed to be unique and
    /// cannot be used to identify a user. In particular, this value cannot be used for log-in
    pub display_name: Option<String>,

    /// A user-customizable link to a [YouTube](https://youtube.com) channel
    pub youtube_channel: Option<String>,
}

impl PartialEq for User {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Display for User {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self.display_name {
            Some(ref dn) => write!(f, "{} '{}' (ID: {})", self.name, dn, self.id),
            None => write!(f, "{} (ID: {})", self.name, self.id),
        }
    }
}

impl User {
    pub fn validate_name(name: &str) -> Result<()> {
        if name.len() < 3 || name != name.trim() {
            return Err(PointercrateError::InvalidUsername)
        }

        Ok(())
    }

    pub fn name(&self) -> &str {
        match self.display_name {
            Some(ref name) => name,
            None => self.name.as_ref(),
        }
    }

    pub fn list_team_member(&self) -> bool {
        self.permissions.implied().contains(Permissions::ListHelper)
    }

    pub fn extended_list_access(&self) -> bool {
        self.permissions.implied().contains(Permissions::ExtendedAccess)
    }
}
