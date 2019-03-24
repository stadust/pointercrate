use super::User;
use crate::{
    context::RequestContext, error::PointercrateError, middleware::auth::Me, operation::Delete,
    schema::members, Result,
};
use diesel::{delete, ExpressionMethods, RunQueryDsl};
use log::info;

impl Delete for User {
    fn delete(self, ctx: RequestContext) -> Result<()> {
        ctx.check_permissions(perms!(Administrator))?;
        ctx.check_if_match(&self)?;

        if let RequestContext::External { user, .. } = ctx {
            if &self == user.unwrap() {
                return Err(PointercrateError::DeleteSelf)
            }
        }

        info!("Deleting user {}", self);

        delete(members::table)
            .filter(members::member_id.eq(self.id))
            .execute(ctx.connection())
            .map(|_| ())
            .map_err(PointercrateError::database)
    }
}

impl Delete for Me {
    fn delete(self, ctx: RequestContext) -> Result<()> {
        ctx.check_if_match(&self)?;

        info!("Self-deleting user {}", self.0);

        delete(members::table)
            .filter(members::member_id.eq(self.0.id))
            .execute(ctx.connection())
            .map(|_| ())
            .map_err(PointercrateError::database)
    }
}
