use crate::auth::TokenAuth;
use log::info;
use pointercrate_core::error::CoreError;
use pointercrate_core_api::{
    error::Result,
    etag::{Precondition, Tagged},
    pagination::pagination_response,
    query::Query,
    response::Response2,
};
use pointercrate_user::{error::UserError, PatchUser, User, UserPagination, ADMINISTRATOR, MODERATOR};
use rocket::{http::Status, serde::json::Json};

#[rocket::get("/")]
pub async fn paginate(mut auth: TokenAuth, data: Query<UserPagination>) -> Result<Response2<Json<Vec<User>>>> {
    let mut pagination = data.0;
    // Rule of thumb: If you can assign permissions, you can see all users that currently have those
    // permissions

    if auth.assignable_permissions().is_empty() {
        return Err(CoreError::Forbidden.into());
    }

    // Pointercrate staff need to be able to see all users, not only those whose permissions they can
    // assign
    if !auth.has_permission(MODERATOR) {
        let assignable_bitmask = auth.assignable_permissions().iter().fold(0x0, |mask, perm| mask | perm.bit());

        pagination.any_permissions = match pagination.any_permissions {
            Some(perms) => Some(perms & assignable_bitmask),
            None => Some(assignable_bitmask),
        }
    }

    Ok(pagination_response("/api/v1/users", pagination, &mut auth.connection).await?)
}

#[rocket::get("/<user_id>")]
pub async fn get_user(mut auth: TokenAuth, user_id: i32) -> Result<Tagged<User>> {
    let user = User::by_id(user_id, &mut auth.connection).await?;

    // We are only allowed to retrieve users who already have permissions we can set.
    if !auth.has_permission(MODERATOR) && !auth.has_permission(ADMINISTRATOR) {
        let can_assign_any = auth.assignable_permissions().iter().any(|perm| user.has_permission(*perm));

        if !can_assign_any {
            // don't leak information about what users exist
            return Err(UserError::UserNotFound { user_id }.into());
        }
    }

    Ok(Tagged(user))
}

#[rocket::patch("/<user_id>", data = "<patch>")]
pub async fn patch_user(mut auth: TokenAuth, precondition: Precondition, user_id: i32, mut patch: Json<PatchUser>) -> Result<Tagged<User>> {
    let user = User::by_id(user_id, &mut auth.connection).await?;

    if !auth.has_permission(MODERATOR) && !auth.has_permission(ADMINISTRATOR) {
        let can_assign_any = auth.assignable_permissions().iter().any(|perm| user.has_permission(*perm));

        if !can_assign_any {
            // don't leak information about what users exist
            return Err(UserError::UserNotFound { user_id }.into());
        }
    }

    if patch.youtube_channel.is_some() || patch.display_name.is_some() {
        auth.require_permission(MODERATOR)?;
    }

    if let Some(ref mut permissions) = patch.permissions {
        let assignable_bitmask = auth.assignable_permissions().iter().fold(0x0, |mask, perm| mask | perm.bit());

        if *permissions & assignable_bitmask != *permissions {
            let unassignable_permissions = (*permissions & assignable_bitmask) ^ *permissions;

            return Err(UserError::PermissionNotAssignable {
                non_assignable: auth.permissions.bits_to_permissions(unassignable_permissions),
            }
            .into());
        }

        info!("assignable permissions are {:b}", assignable_bitmask);
        info!("assigned permissions are {:b}", permissions);
        info!("User currently has permissions {:b}", user.permissions);

        // we clear all the assignable bits in the user's permissions bitstring. Since we already verified
        // that permissions is a subset of assignable_permissions, we can then set the new permissions via
        // simple OR
        *permissions |= user.permissions & !assignable_bitmask;
    }

    if user_id == auth.user.user().id {
        return Err(UserError::PatchSelf.into());
    }

    precondition.require_etag_match(&user)?;

    let user = user.apply_patch(patch.0, &mut auth.connection).await?;

    auth.commit().await?;

    Ok(Tagged(user))
}

#[rocket::delete("/<user_id>")]
pub async fn delete_user(mut auth: TokenAuth, precondition: Precondition, user_id: i32) -> Result<Status> {
    auth.require_permission(ADMINISTRATOR)?;

    if user_id == auth.user.user().id {
        return Err(UserError::DeleteSelf.into());
    }

    let to_delete = User::by_id(user_id, &mut auth.connection).await?;

    precondition.require_etag_match(&to_delete)?;

    to_delete.delete(&mut auth.connection).await?;

    auth.commit().await?;

    Ok(Status::NoContent)
}
