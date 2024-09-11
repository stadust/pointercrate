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

    clnt.patch_player(demon.demon.verifier.id, &helper, serde_json::json!({"subdivision": "SCT"}))
        .await
        .execute()
        .await;

    assert_ne!(nationality_score("GB", &mut connection).await, 0f64);
    assert_eq!(subdivision_score("GB", "ENG", &mut connection).await, 0f64);
    assert_ne!(subdivision_score("GB", "SCT", &mut connection).await, 0f64);

    clnt.patch_player(demon.demon.verifier.id, &helper, serde_json::json!({"nationality": "DE"}))
        .await
        .execute()
        .await;

    assert_eq!(nationality_score("GB", &mut connection).await, 0f64);
    assert_eq!(subdivision_score("GB", "SCT", &mut connection).await, 0f64);
    assert_ne!(nationality_score("DE", &mut connection).await, 0f64);
}

#[sqlx::test(migrations = "../migrations")]
pub async fn test_extended_progress_records_give_no_score(pool: Pool<Postgres>) {
    let (clnt, mut connection) = pointercrate_test::demonlist::setup_rocket(pool).await;

    let helper = pointercrate_test::user::system_user_with_perms(LIST_MODERATOR, &mut *connection).await;
    let player = DatabasePlayer::by_name_or_create("stardust1971", &mut *connection).await.unwrap();

    let list_size = std::env::var("LIST_SIZE").unwrap().parse::<i16>().unwrap();

    let mut last_demon_id = 0;

    for position in 1..=(list_size + 1) {
        last_demon_id = sqlx::query!(
            "INSERT INTO demons (name, position, requirement, verifier, publisher) VALUES ('Bloodbath', $2, 98, $1, $1) RETURNING id",
            player.id,
            position
        )
        .fetch_one(&mut *connection)
        .await
        .unwrap()
        .id;
    }

    let submission = serde_json::json! {{"progress": 99, "demon": last_demon_id, "player": "stardust1972", "video": "https://youtube.com/watch?v=1234567890", "status": "Approved"}};
    let record = clnt
        .post("/api/v1/records", &submission)
        .authorize_as(&helper)
        .expect_status(Status::Ok)
        .get_success_result::<FullRecord>()
        .await;

    let player: FullPlayer = clnt
        .get(format!("/api/v1/players/{}", record.player.id))
        .expect_status(Status::Ok)
        .get_success_result()
        .await;

    assert_eq!(player.player.score, 0.0f64, "Progress record on extended list demon is given score");
}

#[sqlx::test(migrations = "../migrations")]
pub async fn test_score_resets_if_last_record_removed(pool: Pool<Postgres>) {
    let (clnt, mut connection) = pointercrate_test::demonlist::setup_rocket(pool).await;

    let helper = pointercrate_test::user::system_user_with_perms(LIST_MODERATOR, &mut *connection).await;
    let player = DatabasePlayer::by_name_or_create("stardust1971", &mut *connection).await.unwrap();

    let list_size = std::env::var("LIST_SIZE").unwrap().parse::<i16>().unwrap();

    let mut last_demon_id = 0;

    for position in 1..=list_size {
        last_demon_id = sqlx::query!(
            "INSERT INTO demons (name, position, requirement, verifier, publisher) VALUES ('Bloodbath', $2, 98, $1, $1) RETURNING id",
            player.id,
            position
        )
        .fetch_one(&mut *connection)
        .await
        .unwrap()
        .id;
    }

    let submission = serde_json::json! {{"progress": 99, "demon": last_demon_id, "player": "stardust1972", "video": "https://youtube.com/watch?v=1234567890", "status": "Approved"}};
    let record = clnt
        .post("/api/v1/records", &submission)
        .authorize_as(&helper)
        .expect_status(Status::Ok)
        .get_success_result::<FullRecord>()
        .await;

    assert_eq!(record.demon.position, 75);

    let player: FullPlayer = clnt
        .get(format!("/api/v1/players/{}", record.player.id))
        .expect_status(Status::Ok)
        .get_success_result()
        .await;

    assert_ne!(
        player.player.score, 0.0f64,
        "Progress record on final main list demon not giving score"
    );

    // Shift everything down. The demon on which the player has a progress record is now no longer main list, so his score should be updated to 0 now.
    let _ = clnt.add_demon(&helper, "Bloodbath", 1, 100, "stardust1971", "stardust1971").await;

    let record: FullRecord = clnt
        .get(format!("/api/v1/records/{}", record.id))
        .expect_status(Status::Ok)
        .get_success_result()
        .await;

    assert_eq!(record.demon.position, 76);

    let player: FullPlayer = clnt
        .get(format!("/api/v1/players/{}", record.player.id))
        .expect_status(Status::Ok)
        .get_success_result()
        .await;

    assert_eq!(
        player.player.score, 0.0f64,
        "Removal of player's last record did not reset their score to 0"
    );
}
