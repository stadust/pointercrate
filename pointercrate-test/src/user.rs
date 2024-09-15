use crate::TestClient;
use pointercrate_core::{
    permission::{Permission, PermissionsManager},
    pool::PointercratePool,
};
use pointercrate_user::{auth::legacy::Registration, auth::AuthenticatedUser, ADMINISTRATOR, MODERATOR};
use pointercrate_user_pages::account::AccountPageConfig;
use rocket::local::asynchronous::Client;
use sqlx::{pool::PoolConnection, PgConnection, Pool, Postgres};

pub async fn setup_rocket(pool: Pool<Postgres>) -> (TestClient, PoolConnection<Postgres>) {
    let _ = dotenv::dotenv();

    let connection = pool.acquire().await.unwrap();

    let permissions = PermissionsManager::new(vec![MODERATOR, ADMINISTRATOR])
        .assigns(ADMINISTRATOR, MODERATOR)
        .implies(ADMINISTRATOR, MODERATOR);

    let rocket = pointercrate_user_api::setup(rocket::build())
        .manage(PointercratePool::from(pool))
        .manage(permissions)
        .manage(AccountPageConfig::default());

    (TestClient::new(Client::tracked(rocket).await.unwrap()), connection)
}

pub async fn system_user_with_perms(perm: Permission, connection: &mut PgConnection) -> AuthenticatedUser {
    let user = AuthenticatedUser::register(
        Registration {
            name: "Patrick".to_string(),
            password: "bad password".to_string(),
        },
        &mut *connection,
    )
    .await
    .unwrap();

    sqlx::query!(
        "UPDATE members SET permissions = $2::INTEGER::BIT(16) WHERE member_id = $1",
        user.user().id,
        perm.bit() as i16
    )
    .execute(connection)
    .await
    .unwrap();

    user
}

pub async fn add_normal_user(connection: &mut PgConnection) -> AuthenticatedUser {
    AuthenticatedUser::register(
        Registration {
            name: "Patrick".to_string(),
            password: "bad password".to_string(),
        },
        connection,
    )
    .await
    .unwrap()
}
