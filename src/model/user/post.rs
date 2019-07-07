use super::User;
use crate::{
    context::RequestContext, error::PointercrateError, model::Model, operation::Post,
    ratelimit::RatelimitScope, schema::members, Result,
};
use diesel::{insert_into, result::Error, Connection, RunQueryDsl};
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
    password_hash: &'a str,
}

impl Post<Registration> for User {
    fn create_from(mut registration: Registration, ctx: RequestContext) -> Result<User> {
        info!("Creating new user from {:?}", registration);

        User::validate_name(&mut registration.name)?;
        User::validate_password(&mut registration.password)?;

        let connection = ctx.connection();

        ctx.ratelimit(RatelimitScope::SoftRegistration)?;

        connection.transaction(|| {
            match User::by_name(&registration.name).first::<User>(connection) {
                Ok(_) => Err(PointercrateError::NameTaken),
                Err(Error::NotFound) => {
                    ctx.ratelimit(RatelimitScope::Registration)?;

                    info!("Registering new user with name {}", registration.name);

                    let hash = bcrypt::hash(&registration.password, bcrypt::DEFAULT_COST).unwrap();

                    let new = NewUser {
                        name: &registration.name,
                        password_hash: &hash,
                    };

                    insert_into(members::table)
                        .values(&new)
                        .returning(User::selection())
                        .get_result(connection)
                        .map_err(PointercrateError::database)
                },
                Err(err) => Err(PointercrateError::database(err)),
            }
        })
    }
}
