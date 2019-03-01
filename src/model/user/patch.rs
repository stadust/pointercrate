use super::{Permissions, PermissionsSet, User};
use crate::{
    operation::{deserialize_non_optional, deserialize_optional, Hotfix, Patch},
    schema::members,
    Result,
};
use diesel::{ExpressionMethods, PgConnection, RunQueryDsl};
use log::info;
use serde_derive::Deserialize;

make_patch! {
    struct PatchMe {
        password: String,
        display_name: Option<String>,
        youtube_channel: Option<String>,
    }
}

make_patch! {
    struct PatchUser {
        display_name: Option<String>,
        permissions: Permissions,
    }
}

impl Hotfix for PatchMe {
    fn required_permissions(&self) -> PermissionsSet {
        // We can always modify our own account
        PermissionsSet::default()
    }
}

impl Patch<PatchMe> for User {
    fn patch(mut self, mut patch: PatchMe, connection: &PgConnection) -> Result<Self> {
        info!("Patching user {} with {}", self, patch);

        validate!(patch: User::validate_password[password], User::validate_channel[youtube_channel]);

        patch!(self, patch: display_name, youtube_channel);
        patch_with!(self, patch: set_password(&password));

        diesel::update(&self)
            .set((
                members::password_hash.eq(&self.password_hash),
                members::display_name.eq(&self.display_name),
                members::youtube_channel.eq(&self.youtube_channel),
            ))
            .execute(connection)?;

        Ok(self)
    }
}

impl Hotfix for PatchUser {
    fn required_permissions(&self) -> PermissionsSet {
        if let Some(perms) = self.permissions {
            PermissionsSet::one(perms.assignable_from() | Permissions::Moderator)
        } else {
            perms!(Moderator)
        }
    }
}

impl Patch<PatchUser> for User {
    fn patch(mut self, mut patch: PatchUser, connection: &PgConnection) -> Result<Self> {
        info!("Patching user {} with {}", self, patch);

        validate_nullable!(patch: User::validate_name[display_name]);

        patch!(self, patch: display_name);
        patch_with!(self, patch: set_permissions(permissions));

        diesel::update(&self)
            .set((
                members::display_name.eq(&self.display_name),
                members::permissions.eq(&self.permissions),
            ))
            .execute(connection)?;

        Ok(self)
    }
}
