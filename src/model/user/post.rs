use super::User;
use crate::{
    error::PointercrateError,
    model::user::PermissionsSet,
    operation::{Post, PostData},
    schema::members,
    Result,
};
use diesel::{insert_into, result::Error, Connection, PgConnection, RunQueryDsl};
use log::info;
use serde_derive::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Registration {
    pub name: String,
    pub password: String,
}

#[derive(Insertable, Debug)]
#[table_name = "members"]
struct NewUser<'a> {
    name: &'a str,
    password_hash: &'a [u8],
    password_salt: Vec<u8>,
}

impl Post<Registration> for User {
    fn create_from(mut registration: Registration, connection: &PgConnection) -> Result<User> {
        User::validate_name(&mut registration.name)?;
        User::validate_password(&mut registration.password)?;

        connection.transaction(|| {
            match User::by_name(&registration.name).first::<User>(connection) {
                Ok(_) => Err(PointercrateError::NameTaken),
                Err(Error::NotFound) => {
                    info!("Registering new user with name {}", registration.name);

                    let hash = bcrypt::hash(&registration.password, bcrypt::DEFAULT_COST).unwrap();

                    let new = NewUser {
                        name: &registration.name,
                        password_hash: hash.as_bytes(),
                        password_salt: Vec::new(),
                    };

                    insert_into(members::table)
                        .values(&new)
                        .get_result(connection)
                        .map_err(PointercrateError::database)
                },
                Err(err) => Err(PointercrateError::database(err)),
            }
        })
    }
}

impl PostData for Registration {
    fn required_permissions(&self) -> PermissionsSet {
        // Obviously, you cannot have any permissions before registering, as you generally dont have
        // an account (and if you're sending along authorization for an existing account, WHY??)
        PermissionsSet::default()
    }
}
