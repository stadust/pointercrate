use crate::TestClient;
use pointercrate_core::{permission::PermissionsManager, pool::PointercratePool};
use pointercrate_demonlist::{
    player::claim::PlayerClaim, record::RecordStatus, submitter::Submitter, LIST_ADMINISTRATOR, LIST_HELPER, LIST_MODERATOR,
};
use pointercrate_user_pages::account::AccountPageConfig;
use rocket::local::asynchronous::Client;
use sqlx::{pool::PoolConnection, PgConnection, Pool, Postgres};
use std::{net::IpAddr, str::FromStr};

pub async fn setup_rocket(pool: Pool<Postgres>) -> (TestClient, PoolConnection<Postgres>) {
    let _ = dotenv::dotenv();

    let mut connection = pool.acquire().await.unwrap();

    let permissions = PermissionsManager::new(vec![LIST_HELPER, LIST_MODERATOR, LIST_ADMINISTRATOR])
        .assigns(LIST_ADMINISTRATOR, LIST_MODERATOR)
        .implies(LIST_ADMINISTRATOR, LIST_MODERATOR)
        .implies(LIST_MODERATOR, LIST_HELPER);

    let rocket = pointercrate_demonlist_api::setup(rocket::build().manage(PointercratePool::from(pool)))
        .manage(permissions)
        .manage(AccountPageConfig::default());

    // generate some data
    Submitter::create_submitter(IpAddr::from_str("127.0.0.1").unwrap(), &mut connection)
        .await
        .unwrap();

    (TestClient::new(Client::tracked(rocket).await.unwrap()), connection)
}

pub async fn add_demon(
    name: impl Into<String>, position: i16, requirement: i16, verifier_id: i32, publisher_id: i32, connection: &mut PgConnection,
) -> i32 {
    sqlx::query!(
        "INSERT INTO demons (name, position, requirement, verifier, publisher) VALUES ($1::TEXT::CITEXT, $2, $3, $4, $5) RETURNING id",
        name.into(),
        position,
        requirement,
        verifier_id,
        publisher_id
    )
    .fetch_one(&mut *connection)
    .await
    .unwrap()
    .id
}

pub async fn put_claim(user_id: i32, player_id: i32, verified: bool, lock_submissions: bool, connection: &mut PgConnection) -> PlayerClaim {
    sqlx::query!(
        "INSERT INTO player_claims (member_id, player_id, verified, lock_submissions) VALUES ($1, $2, $3, $4)",
        user_id,
        player_id,
        verified,
        lock_submissions
    )
    .execute(connection)
    .await
    .unwrap();

    PlayerClaim {
        user_id,
        player_id,
        verified,
        lock_submissions,
    }
}

pub async fn add_simple_record(progress: i16, player: i32, demon: i32, status: RecordStatus, connection: &mut PgConnection) -> i32 {
    let system_sub = Submitter::by_ip(IpAddr::from_str("127.0.0.1").unwrap(), &mut *connection)
        .await
        .unwrap()
        .unwrap();

    sqlx::query!(
        "INSERT INTO records (progress, status_, player, submitter, demon, video) VALUES ($1, $2::text::record_status, $3, $4, $5, NULL) \
         RETURNING id",
        progress,
        status.to_sql(),
        player,
        system_sub.id,
        demon
    )
    .fetch_one(&mut *connection)
    .await
    .unwrap()
    .id
}
