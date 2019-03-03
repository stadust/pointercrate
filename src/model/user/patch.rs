use super::{Permissions, PermissionsSet, User};
use crate::{
    middleware::auth::Me,
    operation::{deserialize_non_optional, deserialize_optional, Patch},
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

impl Patch<PatchMe> for Me {
    fn patch(mut self, mut patch: PatchMe, connection: &PgConnection) -> Result<Self> {
        //info!("Patching user {} with {}", self, patch);

        validate!(patch: User::validate_password[password], User::validate_channel[youtube_channel]);

        patch!(self.0, patch: display_name, youtube_channel);
        patch_with!(self.0, patch: set_password(&password));

        diesel::update(&self.0)
            .set((
                members::password_hash.eq(&self.0.password_hash),
                members::display_name.eq(&self.0.display_name),
                members::youtube_channel.eq(&self.0.youtube_channel),
            ))
            .execute(connection)?;

        Ok(self)
    }

    fn permissions_for(&self, _: &PatchMe) -> PermissionsSet {
        // We can always modify our own account
        PermissionsSet::default()
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
