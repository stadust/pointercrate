use super::User;
use crate::{
    context::RequestContext, error::PointercrateError, middleware::auth::Me, operation::Get,
    permissions::Permissions, Result,
};
use diesel::{result::Error, RunQueryDsl};

impl Get<i32> for User {
    fn get(id: i32, ctx: RequestContext) -> Result<User> {
        ctx.check_permissions(perms!(Moderator or Administrator))?;

        match User::by_id(id).first(ctx.connection()) {
            Ok(user) => Ok(user),
            Err(Error::NotFound) =>
                Err(PointercrateError::ModelNotFound {
                    model: "User",
                    identified_by: id.to_string(),
                }),
            Err(err) => Err(PointercrateError::database(err)),
        }
    }
}

impl Get<String> for User {
    fn get(name: String, ctx: RequestContext) -> Result<User> {
        ctx.check_permissions(perms!(Moderator or Administrator))?;

        match User::by_name(&name).first(ctx.connection()) {
            Ok(user) => Ok(user),
            Err(Error::NotFound) =>
                Err(PointercrateError::ModelNotFound {
                    model: "User",
                    identified_by: name,
                }),
            Err(err) => Err(PointercrateError::database(err)),
        }
    }
}

impl Get<Permissions> for Vec<User> {
    fn get(perms: Permissions, ctx: RequestContext) -> Result<Vec<User>> {
        ctx.check_permissions(perms!(Administrator))?;

        Ok(User::by_permissions(perms).load(ctx.connection())?)
    }
}

impl Get<Me> for Me {
    fn get(me: Me, _ctx: RequestContext) -> Result<Me> {
        Ok(me)
    }
}
