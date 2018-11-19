use super::{Permissions, User};
use crate::{
    error::PointercrateError,
    operation::{deserialize_patch, Hotfix, Patch, PatchField},
    schema::members,
    Result,
};
use diesel::{ExpressionMethods, PgConnection, RunQueryDsl};
use serde_derive::Deserialize;

make_patch! {
    struct PatchMe {
        password: String,
        display_name: String,
        youtube_channel: String
    }
}

make_patch! {
    struct PatchUser {
        display_name: String,
        permissions: Permissions
    }
}

impl Hotfix for PatchMe {}

impl Patch<PatchMe> for User {
    fn patch(mut self, patch: PatchMe, connection: &PgConnection) -> Result<Self> {
        if let PatchField::Some(ref password) = patch.password {
            if password.len() < 10 {
                return Err(PointercrateError::InvalidPassword)
            }
        }

        patch_not_null!(self, patch, password, set_password);
        patch!(self, patch, display_name);
        patch!(self, patch, youtube_channel);

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
        if let PatchField::Some(perms) = self.permissions {
            perms.assignable_from() | Permissions::Moderator
        } else {
            Permissions::Moderator
        }
    }
}

impl Patch<PatchUser> for User {
    fn patch(mut self, patch: PatchUser, connection: &PgConnection) -> Result<Self> {
        if let PatchField::Some(ref display_name) = patch.display_name {
            if display_name.len() < 3 || display_name != display_name.trim() {
                return Err(PointercrateError::InvalidUsername)
            }
        }

        patch!(self, patch, display_name);
        patch_not_null!(self, patch, permissions, *set_permissions);

        diesel::update(&self)
            .set((
                members::display_name.eq(&self.display_name),
                members::permissions.eq(&self.permissions),
            ))
            .execute(connection)?;

        Ok(self)
    }
}
