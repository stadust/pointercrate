#[cfg(feature = "legacy_accounts")]
pub use register::Registration;

#[cfg(feature = "legacy_accounts")]
mod register {
    use crate::{
        auth::{AuthenticatedUser, AuthenticationType, PasswordOrBrowser},
        error::UserError,
        Result, User,
    };
    use serde::{Deserialize, Serialize};
    use sqlx::PgConnection;

    #[derive(Deserialize, Serialize)]
    pub struct Registration {
        pub name: String,
        pub password: String,
    }

    impl AuthenticatedUser<PasswordOrBrowser> {
        pub async fn register(registration: Registration, connection: &mut PgConnection) -> Result<Self> {
            log::info!("Attempting registration of new user under name {}", registration.name);

            log::trace!("Registration request is formally correct");

            match User::by_name(&registration.name, connection).await {
                Ok(_) => Err(UserError::NameTaken),
                Err(UserError::UserNotFoundName { .. }) => {
                    let hash = bcrypt::hash(&registration.password, bcrypt::DEFAULT_COST).unwrap();

                    let id = sqlx::query!(
                        "INSERT INTO members (name, password_hash) VALUES ($1, $2) RETURNING member_id",
                        registration.name,
                        hash
                    )
                    .fetch_one(connection)
                    .await?
                    .member_id;

                    log::info!("Newly registered user with name {} has been assigned ID {}", registration.name, id);

                    Ok(AuthenticatedUser {
                        gen: 0,
                        auth_type: AuthenticationType::legacy(
                            User {
                                id,
                                name: registration.name,
                                permissions: 0,
                                display_name: None,
                                youtube_channel: None,
                            },
                            hash,
                        ),
                        auth_artifact: PasswordOrBrowser(true),
                    })
                },
                Err(err) => Err(err),
            }
        }
    }
}
