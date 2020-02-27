//! Module for general administrative user actions. See [`auth`] for account related user actions
//!
//! Includes:
//! * Deleting other accounts
//! * Modifying other people's accounts (assign permissions, change offensive names, etc)
//! * Querying account information

pub use self::{
    auth::{AuthenticatedUser, Authorization, PatchMe, Registration},
    paginate::UserPagination,
    patch::PatchUser,
};
use crate::{error::PointercrateError, permissions::Permissions, Result};
use serde::Serialize;
use sqlx::PgConnection;
use std::{
    fmt::{Display, Formatter},
    hash::Hash,
};

mod auth;
mod delete;
mod get;
mod paginate;
mod patch;

// TODO: impl the nationality stuff already in the database
/// Model representing a user in the database
#[derive(Debug, Serialize, Hash, Eq, PartialEq)]
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

impl Display for User {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self.display_name {
            Some(ref dn) => write!(f, "{} '{}' (ID: {})", self.name, dn, self.id),
            None => write!(f, "{} (ID: {})", self.name, self.id),
        }
    }
}

impl User {
    pub fn has_permission(&self, perm: Permissions) -> bool {
        self.permissions.implied().contains(perm)
    }

    pub fn require_permissions(&self, perm: Permissions) -> Result<()> {
        if !self.has_permission(perm) {
            return Err(PointercrateError::MissingPermissions { required: perm })
        }

        Ok(())
    }

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

    /// Gets the maximal and minimal member id currently in use
    ///
    /// The returned tuple is of the form (max, min)
    pub async fn extremal_member_ids(connection: &mut PgConnection) -> Result<(i32, i32)> {
        let row = sqlx::query!("SELECT MAX(member_id) AS max_id, MIN(member_id) AS min_id FROM members")
            .fetch_one(connection)
            .await?; // FIXME: crashes on empty table
        Ok((row.max_id, row.min_id))
    }
}
