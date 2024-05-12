//! Module containing all score related test cases (because I suspect over time there will be quite a few)

use pointercrate_core::etag::Taggable;
use pointercrate_demonlist::{
    player::{DatabasePlayer, FullPlayer},
    record::FullRecord,
    LIST_MODERATOR,
};
use rocket::http::Status;
use sqlx::{PgConnection, Pool, Postgres};

#[sqlx::test(migrations = "../migrations")]
pub async fn test_score_update_on_record_update(pool: Pool<Postgres>) {
    let (clnt, mut connection) = pointercrate_test::demonlist::setup_rocket(pool).await;

    let helper = pointercrate_test::user::system_user_with_perms(LIST_MODERATOR, &mut *connection).await;
    let player = DatabasePlayer::by_name_or_create("stardust1971", &mut *connection).await.unwrap();
    let demon = clnt.add_demon(&helper, "Bloodbath", 1, 100, "stardust1972", "stardust1972").await;

    let submission = serde_json::json! {{"progress": 100, "demon": demon.demon.base.id, "player": "stardust1971", "video": "https://youtube.com/watch?v=1234567890", "status": "Approved"}};

    let record = clnt
        .post("/api/v1/records", &submission)
        .authorize_as(&helper)
        .expect_status(Status::Ok)
        .get_success_result::<FullRecord>()
        .await;

    let player: FullPlayer = clnt
        .get(format!("/api/v1/players/{}", player.id))
        .expect_status(Status::Ok)
        .get_success_result()
        .await;

    assert_ne!(player.player.score, 0.0f64, "Adding approved record failed to give player score");

    clnt.patch(
        format!("/api/v1/records/{}/", record.id),
        &serde_json::json!({"status": "Rejected"}),
    )
    .authorize_as(&helper)
    .header("If-Match", record.etag_string())
    .expect_status(Status::Ok)
    .execute()
    .await;

    let player: FullPlayer = clnt
        .get(format!("/api/v1/players/{}", player.player.base.id))
        .expect_status(Status::Ok)
        .get_success_result()
        .await;

    assert_eq!(player.player.score, 0.0f64, "Rejecting record failed to remove player score");
}

#[sqlx::test(migrations = "../migrations")]
pub async fn test_verifications_give_score(pool: Pool<Postgres>) {
    let (clnt, mut connection) = pointercrate_test::demonlist::setup_rocket(pool).await;

    let helper = pointercrate_test::user::system_user_with_perms(LIST_MODERATOR, &mut *connection).await;
    let demon = clnt.add_demon(&helper, "Bloodbath", 1, 100, "stardust1971", "stardust1971").await;

    let player: FullPlayer = clnt
        .get(format!("/api/v1/players/{}", demon.demon.verifier.id))
        .expect_status(Status::Ok)
        .get_success_result()
        .await;

    assert_ne!(player.player.score, 0.0f64);
}

async fn nationality_score(iso_country_code: &str, connection: &mut PgConnection) -> f64 {
    sqlx::query!("SELECT score FROM nationalities WHERE iso_country_code = $1", iso_country_code)
        .fetch_one(&mut *connection)
        .await
        .unwrap()
        .score
}

async fn subdivision_score(nation: &str, iso_code: &str, connection: &mut PgConnection) -> f64 {
    sqlx::query!(
        "SELECT score FROM subdivisions WHERE nation = $1 AND iso_code = $2",
        nation,
        iso_code
    )
    .fetch_one(&mut *connection)
    .await
    .unwrap()
    .score
}

#[sqlx::test(migrations = "../migrations")]
pub async fn test_player_score_reflects_to_nationality(pool: Pool<Postgres>) {
    let (clnt, mut connection) = pointercrate_test::demonlist::setup_rocket(pool).await;

    let helper = pointercrate_test::user::system_user_with_perms(LIST_MODERATOR, &mut *connection).await;
    let demon = clnt.add_demon(&helper, "Bloodbath", 1, 100, "stardust1971", "stardust1971").await;

    clnt.patch_player(
        demon.demon.verifier.id,
        &helper,
        serde_json::json!({"nationality": "GB", "subdivision": "ENG"}),
    )
    .await
    .execute()
    .await;

    assert_ne!(nationality_score("GB", &mut connection).await, 0f64);
    assert_ne!(subdivision_score("GB", "ENG", &mut connection).await, 0f64);

    clnt.patch_player(
        demon.demon.verifier.id,
        &helper,
        serde_json::json!({"subdivision": "SCT"}),
    )
    .await
    .execute()
    .await;

    assert_ne!(nationality_score("GB", &mut connection).await, 0f64);
    assert_eq!(subdivision_score("GB", "ENG", &mut connection).await, 0f64);
    assert_ne!(subdivision_score("GB", "SCT", &mut connection).await, 0f64);

    clnt.patch_player(
        demon.demon.verifier.id,
        &helper,
        serde_json::json!({"nationality": "DE"}),
    )
    .await
    .execute()
    .await;

    assert_eq!(nationality_score("GB", &mut connection).await, 0f64);
    assert_eq!(subdivision_score("GB", "SCT", &mut connection).await, 0f64);
    assert_ne!(nationality_score("DE", &mut connection).await, 0f64);
}
