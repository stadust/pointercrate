use super::AuthenticatedUser;
use crate::{error::PointercrateError, model::user::User, permissions::Permissions, Result};
use log::info;
use serde::Deserialize;
use sqlx::PgConnection;

#[derive(Deserialize)]
pub struct Registration {
    pub name: String,
    pub password: String,
}

impl AuthenticatedUser {
    pub async fn register(registration: Registration, connection: &mut PgConnection) -> Result<AuthenticatedUser> {
        info!("Attempting registration of new user under name {}", registration.name);

        Self::validate_password(&registration.password)?;
        User::validate_name(&registration.name)?;

        match User::by_name(&registration.name, connection).await {
            Ok(_) => Err(PointercrateError::NameTaken),
            Err(PointercrateError::ModelNotFound { .. }) => {
                let hash = bcrypt::hash(&registration.password, bcrypt::DEFAULT_COST).unwrap();

                let id = sqlx::query!(
                    "INSERT INTO members (name, password_hash) VALUES ($1, $2) RETURNING member_id",
                    registration.name,
                    hash
                )
                .fetch_one(connection)
                .await?
                .member_id;

                Ok(AuthenticatedUser {
                    user: User {
                        id,
                        name: registration.name,
                        permissions: Permissions::empty(),
                        display_name: None,
                        youtube_channel: None,
                    },
                    password_hash: hash,
                })
            },
            Err(err) => Err(err),
        }
    }
}
