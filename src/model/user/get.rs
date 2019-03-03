use super::User;
use crate::{
    error::PointercrateError,
    middleware::auth::Me,
    operation::Get,
    permissions::{self, AccessRestrictions, Permissions},
    Result,
};
use diesel::{result::Error, PgConnection, RunQueryDsl};

impl Get<i32> for User {
    fn get(id: i32, connection: &PgConnection) -> Result<User> {
        match User::by_id(id).first(connection) {
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
    fn get(name: String, connection: &PgConnection) -> Result<User> {
        match User::by_name(&name).first(connection) {
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
    fn get(perms: Permissions, connection: &PgConnection) -> Result<Vec<User>> {
        Ok(User::by_permissions(perms).load(connection)?)
    }
}

// TODO: check jurisdiction in `access()` and `page_access()`
impl AccessRestrictions for User {
    fn pre_access(user: Option<&User>) -> Result<()> {
        permissions::demand(perms!(Moderator or Administrator), user)
    }

    fn pre_page_access(user: Option<&User>) -> Result<()> {
        permissions::demand(perms!(Administrator), user)
    }

    fn pre_delete(&self, user: Option<&User>) -> Result<()> {
        permissions::demand(perms!(Administrator), user)?;

        if self.id == user.unwrap().id {
            return Err(PointercrateError::DeleteSelf)
        }

        Ok(())
    }

    fn pre_patch(&self, user: Option<&User>) -> Result<()> {
        if let Some(user) = user {
            if self.id == user.id {
                return Err(PointercrateError::PatchSelf)
            }
        }

        Ok(())
    }
}

impl Get<Me> for Me {
    fn get(me: Me, _: &PgConnection) -> Result<Me> {
        Ok(me)
    }
}

impl AccessRestrictions for Me {}
