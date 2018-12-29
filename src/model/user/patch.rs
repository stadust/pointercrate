use super::{Permissions, User};
use crate::{
    operation::{deserialize_non_optional, deserialize_optional, Hotfix, Patch},
    schema::members,
    Result,
};
use diesel::{ExpressionMethods, PgConnection, RunQueryDsl};
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

impl Hotfix for PatchMe {}

impl Patch<PatchMe> for User {
    fn patch(mut self, mut patch: PatchMe, connection: &PgConnection) -> Result<Self> {
        validate!(patch: User::validate_password[password]);
        //TODO: youtube channel url validation

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
    fn required_permissions(&self) -> Permissions {
        if let Some(perms) = self.permissions {
            perms.assignable_from() | Permissions::Moderator
        } else {
            Permissions::Moderator
        }
    }
}

impl Patch<PatchUser> for User {
    fn patch(mut self, mut patch: PatchUser, connection: &PgConnection) -> Result<Self> {
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
