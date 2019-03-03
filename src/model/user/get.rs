use super::User;
use crate::{
    error::PointercrateError,
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

// TODO: check jurisdiction in `access()`
impl AccessRestrictions for User {
    fn pre_access(user: Option<&User>) -> Result<()> {
        permissions::demand(perms!(Moderator or Administrator), user)
    }
}
