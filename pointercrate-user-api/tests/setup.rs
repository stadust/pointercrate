use pointercrate_core::{permission::PermissionsManager, pool::PointercratePool};
use pointercrate_user::{AuthenticatedUser, Registration, ADMINISTRATOR, MODERATOR};
use pointercrate_user_pages::account::AccountPageConfig;
use rocket::local::asynchronous::Client;
use sqlx::{Pool, pool::PoolConnection, Postgres};

pub async fn setup(pool: Pool<Postgres>) -> Client {
    dotenv::dotenv().unwrap();

    let permissions = PermissionsManager::new(vec![MODERATOR, ADMINISTRATOR])
        .assigns(ADMINISTRATOR, MODERATOR)
        .implies(ADMINISTRATOR, MODERATOR);

    let rocket = pointercrate_user_api::setup(rocket::build())
        .manage(PointercratePool::from(pool))
        .manage(permissions)
        .manage(AccountPageConfig::default());

    Client::tracked(rocket).await.unwrap()
}

pub async fn setup_with_admin_user(pool: Pool<Postgres>) -> (Client, AuthenticatedUser) {
    let mut connection = pool.acquire().await.unwrap();
    let client = setup(pool).await;

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

    (client, user)
}
