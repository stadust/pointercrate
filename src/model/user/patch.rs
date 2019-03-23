use super::{Permissions, PermissionsSet, User};
use crate::{
    context::RequestContext,
    middleware::auth::Me,
    operation::{deserialize_non_optional, deserialize_optional, Patch},
    schema::members,
    Result,
};
use diesel::{ExpressionMethods, PgConnection, RunQueryDsl};
use log::info;
use serde_derive::Deserialize;
use crate::error::PointercrateError;

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
    fn patch(
        mut self, mut patch: PatchMe, ctx: RequestContext, connection: &PgConnection,
    ) -> Result<Self> {
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
}

impl Patch<PatchUser> for User {
    fn patch(
        mut self, mut patch: PatchUser, ctx: RequestContext, connection: &PgConnection,
    ) -> Result<Self> {
        match patch {
            PatchUser {
                display_name: None,
                permissions: None,
            } => ctx.check_permissions(perms!(Administrator))?,

            PatchUser {
                display_name: Some(_),
                permissions: None,
            } => ctx.check_permissions(perms!(Moderator))?,

            PatchUser {
                display_name: None,
                permissions: Some(ref perms),
            } => ctx.check_permissions((*perms ^ self.permissions()).assignable_from())?,

            PatchUser {
                display_name: Some(_),
                permissions: Some(ref perms),
            } =>
                ctx.check_permissions(
                    (*perms ^ self.permissions())
                        .assignable_from()
                        .cross(&PermissionsSet::one(Permissions::Moderator)),
                )?,
        }

        if let RequestContext::External { user, .. } = ctx {
            if &self == user.unwrap() {
                return Err(PointercrateError::PatchSelf)
            }
        }

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
