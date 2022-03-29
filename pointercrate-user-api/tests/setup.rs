use pointercrate_core::{permission::PermissionsManager, pool::PointercratePool};
use pointercrate_user::{AuthenticatedUser, Registration, User, ADMINISTRATOR, MODERATOR};
use pointercrate_user_pages::account::AccountPageConfig;
use rocket::{local::asynchronous::Client, Build, Rocket};
use sqlx::{pool::PoolConnection, Pool, Postgres};

pub async fn setup() -> (Client, PoolConnection<Postgres>) {
    let pool = PointercratePool::init().await;
    let connection = pool.connection().await.unwrap();

    // reset test database
    sqlx::query!("TRUNCATE TABLE members CASCADE")
        .execute(&mut pool.connection().await.unwrap())
        .await
        .unwrap();

    let permissions = PermissionsManager::new(vec![MODERATOR, ADMINISTRATOR])
        .assigns(ADMINISTRATOR, MODERATOR)
        .implies(ADMINISTRATOR, MODERATOR);

    let rocket = pointercrate_user_api::setup(rocket::build())
        .manage(pool)
        .manage(permissions)
        .manage(AccountPageConfig::default());

    (Client::tracked(rocket).await.unwrap(), connection)
}

pub async fn setup_with_admin_user() -> (Client, AuthenticatedUser, PoolConnection<Postgres>) {
    let (client, mut connection) = setup().await;

    let user = AuthenticatedUser::register(
        Registration {
            name: "Patrick".to_string(),
            password: "bad password".to_string(),
        },
        &mut connection,
    )
    .await
    .unwrap();

    sqlx::query!(
        "UPDATE members SET permissions = $2::INTEGER::BIT(16) WHERE member_id = $1",
        user.inner().id,
        ADMINISTRATOR.bit() as i16
    )
    .execute(&mut connection)
    .await
    .unwrap();

    (client, user, connection)
}
