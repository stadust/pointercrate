use pointercrate_core::{permission::PermissionsManager, pool::PointercratePool};
use pointercrate_user::{ADMINISTRATOR, MODERATOR};
use pointercrate_user_pages::account::AccountPageConfig;
use rocket::{local::asynchronous::Client, Build, Rocket};

pub async fn setup() -> Client {
    let pool = PointercratePool::init().await;

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

    Client::tracked(rocket).await.unwrap()
}
