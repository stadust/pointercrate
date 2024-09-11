use crate::{TestClient, TestRequest};
use pointercrate_core::etag::Taggable;
use pointercrate_core::{permission::PermissionsManager, pool::PointercratePool};
use pointercrate_demonlist::demon::FullDemon;
use pointercrate_demonlist::{
    player::{claim::PlayerClaim, FullPlayer},
    record::RecordStatus,
    submitter::Submitter,
    LIST_ADMINISTRATOR, LIST_HELPER, LIST_MODERATOR,
};
use pointercrate_user::auth::AuthenticatedUser;
use pointercrate_user_pages::account::AccountPageConfig;
use rocket::{http::Status, local::asynchronous::Client};
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
    Submitter::create_submitter(IpAddr::from_str("127.0.0.1").unwrap(), &mut *connection)
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
        "INSERT INTO records (progress, status_, player, submitter, demon, video, raw_footage) VALUES ($1, $2::text::record_status, $3, $4, $5, NULL, NULL) \
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

impl TestClient {
    pub async fn patch_player(&self, player_id: i32, auth_context: &AuthenticatedUser, patch: serde_json::Value) -> TestRequest {
        let player: FullPlayer = self
            .get(format!("/api/v1/players/{}/", player_id))
            .expect_status(Status::Ok)
            .get_success_result()
            .await;

        self.patch(format!("/api/v1/players/{}/", player_id), &patch)
            .authorize_as(&auth_context)
            .header("If-Match", player.etag_string())
            .expect_status(Status::Ok)
    }

    pub async fn add_demon(
        &self, auth_context: &AuthenticatedUser, name: impl Into<String>, position: i16, requirement: i16, verifier: impl Into<String>,
        publisher: impl Into<String>,
    ) -> FullDemon {
        self.post("/api/v2/demons/", &serde_json::json!({"name": name.into(), "position": position, "requirement": requirement, "verifier": verifier.into(), "publisher": publisher.into(), "creators": []}))
            .expect_status(Status::Created)
            .authorize_as(&auth_context)
            .get_success_result()
            .await
    }
}
