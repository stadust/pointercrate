use super::User;
use crate::{error::PointercrateError, operation::Post, schema::members, Result};
use diesel::{insert_into, result::Error, PgConnection, RunQueryDsl};
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
    fn create_from(registration: Registration, connection: &PgConnection) -> Result<User> {
        if registration.name.len() < 3 || registration.name != registration.name.trim() {
            return Err(PointercrateError::InvalidUsername)
        }

        if registration.password.len() < 10 {
            return Err(PointercrateError::InvalidPassword)
        }

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
    }
}
