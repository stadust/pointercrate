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
        //info!("Patching user {} with {}", self, patch);

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

    fn permissions_for(&self, _: &PatchMe) -> PermissionsSet {
        // We can always modify our own account
        PermissionsSet::default()
    }
}

impl Hotfix for PatchUser {
    fn required_permissions(&self) -> PermissionsSet {
        // FIXME: Ideally, here we return permissions based on what is supposed to be patched and
        // which of the user's permissions are affected by the patch. For this we need to calculate
        // the difference between the targets current permissions and the applied patch (XOR of the
        // bit strings). This is not possible with our current implementation. The commented out
        // implementation has the severe flaw that every can revoke arbitrary permissions from
        // anyone else.
        match self.permissions {
            Some(perms) if perms & Permissions::Administrator != Permissions::empty() =>
                perms!(ItIsImpossibleToGainThisPermission),
            _ => perms!(Administrator),
        }
        /*if let Some(perms) = self.permissions {
            if self.display_name.is_none() {
                PermissionsSet::one(perms.assignable_from())
            } else {
                PermissionsSet::one(perms.assignable_from() | Permissions::Moderator)
            }
        } else {
            perms!(Moderator)
        }*/
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

    fn permissions_for(&self, patch: &PatchUser) -> PermissionsSet {
        match patch {
            PatchUser {
                display_name: None,
                permissions: None,
            } => perms!(Administrator),

            PatchUser {
                display_name: Some(_),
                permissions: None,
            } => perms!(Moderator),

            PatchUser {
                display_name: None,
                permissions: Some(perms),
            } => PermissionsSet::one((*perms ^ self.permissions()).assignable_from()),

            PatchUser {
                display_name: Some(_),
                permissions: Some(perms),
            } =>
                PermissionsSet::one(
                    (*perms ^ self.permissions()).assignable_from() | Permissions::Moderator,
                ),
        }
    }
}
