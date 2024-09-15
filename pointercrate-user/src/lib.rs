//! Module for general administrative user actions. See [`auth`] for account related user actions
//!
//! Includes:
//! * Deleting other accounts
//! * Modifying other people's accounts (assign permissions, change offensive names, etc)
//! * Querying account information

pub use self::{paginate::UserPagination, patch::PatchUser};
use crate::error::{Result, UserError};
use pointercrate_core::{
    etag::Taggable,
    permission::{Permission, PermissionsManager},
};
use serde::Serialize;
use std::{
    fmt::{Display, Formatter},
    hash::Hash,
};

#[macro_use]
mod get;
pub mod auth;
mod delete;
pub mod error;
mod paginate;
mod patch;
mod video;

pub const ADMINISTRATOR: Permission = Permission::new("Administrator", 0x4000);
pub const MODERATOR: Permission = Permission::new("Moderator", 0x2000);

pub fn default_permissions_manager() -> PermissionsManager {
    PermissionsManager::new(vec![ADMINISTRATOR, MODERATOR])
        .assigns(ADMINISTRATOR, MODERATOR)
        .implies(ADMINISTRATOR, MODERATOR)
}

/// Model representing a user in the database
#[derive(Debug, Serialize, Hash, Eq, PartialEq)]
pub struct User {
    /// The [`User`]'s unique ID. This is used to identify users and cannot be changed.
    pub id: i32,

    /// The [`User`]'s unique username. This is used to log-in and cannot be changed.
    pub name: String,

    pub permissions: u16,

    /// A user-customizable name for each [`User`].
    ///
    /// If set to anything other than [`None`], the value set here will be displayed everywhere the
    /// username would be displayed otherwise. This value is not guaranteed to be unique and
    /// cannot be used to identify a user. In particular, this value cannot be used for log-in
    pub display_name: Option<String>,

    /// A user-customizable link to a [YouTube](https://youtube.com) channel
    pub youtube_channel: Option<String>,
}

impl Taggable for User {}

impl Display for User {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self.display_name {
            Some(ref dn) => write!(f, "{} '{}' (ID: {})", self.name, dn, self.id),
            None => write!(f, "{} (ID: {})", self.name, self.id),
        }
    }
}

impl User {
    pub fn has_permission(&self, permission: Permission) -> bool {
        self.has_permissions(permission.bit())
    }

    pub fn has_permissions(&self, perms: u16) -> bool {
        self.permissions & perms == perms
    }

    pub fn has_any_permissions(&self, perms: impl Iterator<Item = u16>) -> bool {
        perms.into_iter().any(|perm| self.has_permissions(perm))
    }

    pub fn validate_name(name: &str) -> Result<()> {
        if name.len() < 3 || name != name.trim() {
            return Err(UserError::InvalidUsername);
        }

        Ok(())
    }

    pub fn name(&self) -> &str {
        match self.display_name {
            Some(ref name) => name,
            None => self.name.as_ref(),
        }
    }
}
