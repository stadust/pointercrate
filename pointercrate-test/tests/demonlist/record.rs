use pointercrate_core::error::PointercrateError;
use pointercrate_core::etag::Taggable;
use pointercrate_demonlist::{
    error::DemonlistError,
    player::{DatabasePlayer, FullPlayer},
    record::{note::Note, FullRecord, RecordStatus},
    LIST_HELPER, LIST_MODERATOR,
};
use pointercrate_test::{demonlist::add_simple_record, user::system_user_with_perms};
use rocket::http::Status;
use sqlx::{PgConnection, Pool, Postgres};

#[sqlx::test(migrations = "../migrations")]
async fn paginate_records_unauthorized(pool: Pool<Postgres>) {
    let (clnt, mut connection) = pointercrate_test::demonlist::setup_rocket(pool).await;

    let (p1, r1, _r2, _r3) = setup_pagination_tests(&mut *connection).await;

    let json: Vec<serde_json::Value> = clnt
        .get(format!("/api/v1/records/?player={}", p1))
        .expect_status(Status::Ok)
        .get_result()
        .await;

    assert_eq!(json.len(), 1);
    assert_eq!(json[0]["id"].as_i64(), Some(r1 as i64));
}

#[sqlx::test(migrations = "../migrations")]
async fn paginate_records_with_verified_claim(pool: Pool<Postgres>) {
    let (clnt, mut connection) = pointercrate_test::demonlist::setup_rocket(pool).await;

    let (p1, r1, r2, _r3) = setup_pagination_tests(&mut *connection).await;
    let user = pointercrate_test::user::add_normal_user(&mut *connection).await;

    pointercrate_test::demonlist::put_claim(user.inner().id, p1, true, false, &mut *connection).await;

    let json: Vec<serde_json::Value> = clnt
        .get(format!("/api/v1/records/?player={}", p1))
        .authorize_as(&user)
        .get_result()
        .await;

    assert_eq!(json.len(), 2);
    assert_eq!(json[0]["id"].as_i64(), Some(r1 as i64));
    assert_eq!(json[1]["id"].as_i64(), Some(r2 as i64));
}

#[sqlx::test(migrations = "../migrations")]
async fn paginate_records_with_unverified_claim(pool: Pool<Postgres>) {
    let (clnt, mut connection) = pointercrate_test::demonlist::setup_rocket(pool).await;

    let (p1, r1, _r2, _r3) = setup_pagination_tests(&mut *connection).await;
    let user = pointercrate_test::user::add_normal_user(&mut *connection).await;

    pointercrate_test::demonlist::put_claim(user.inner().id, p1, false, false, &mut *connection).await;

    let json: Vec<serde_json::Value> = clnt
        .get(format!("/api/v1/records/?player={}", p1))
        .authorize_as(&user)
        .get_result()
        .await;

    assert_eq!(json.len(), 1);
    assert_eq!(json[0]["id"].as_i64(), Some(r1 as i64));
}

#[sqlx::test(migrations = "../migrations")]
async fn paginate_records_with_verified_claim_wrong_player(pool: Pool<Postgres>) {
    let (clnt, mut connection) = pointercrate_test::demonlist::setup_rocket(pool).await;

    let (p1, _r1, _r2, _r3) = setup_pagination_tests(&mut *connection).await;
    let user = pointercrate_test::user::add_normal_user(&mut *connection).await;

    pointercrate_test::demonlist::put_claim(user.inner().id, p1, true, false, &mut *connection).await;

    let json: Vec<serde_json::Value> = clnt.get("/api/v1/records/?player=2").authorize_as(&user).get_result().await;

    assert_eq!(json.len(), 0);
}

async fn setup_pagination_tests(connection: &mut PgConnection) -> (i32, i32, i32, i32) {
    let player1 = DatabasePlayer::by_name_or_create("stardust1971", connection).await.unwrap();
    let player2 = DatabasePlayer::by_name_or_create("stardust1972", connection).await.unwrap();

    let demon1 = pointercrate_test::demonlist::add_demon("Bloodbath", 1, 87, player1.id, player1.id, connection).await;
    let demon2 = pointercrate_test::demonlist::add_demon("Bloodlust", 2, 53, player1.id, player1.id, connection).await;

    let r1 = pointercrate_test::demonlist::add_simple_record(100, player1.id, demon1, RecordStatus::Approved, connection).await;
    let r2 = pointercrate_test::demonlist::add_simple_record(70, player1.id, demon2, RecordStatus::Rejected, connection).await;
    let r3 = pointercrate_test::demonlist::add_simple_record(100, player2.id, demon2, RecordStatus::Rejected, connection).await;

    (player1.id, r1, r2, r3)
}

#[sqlx::test(migrations = "../migrations")]
async fn unauthed_submit_for_player_with_locked_submission(pool: Pool<Postgres>) {
    let (clnt, mut connection) = pointercrate_test::demonlist::setup_rocket(pool).await;

    let user = pointercrate_test::user::add_normal_user(&mut *connection).await;
    let player1 = DatabasePlayer::by_name_or_create("stardust1971", &mut *connection).await.unwrap();
    let demon1 = pointercrate_test::demonlist::add_demon("Bloodbath", 1, 87, player1.id, player1.id, &mut *connection).await;

    pointercrate_test::demonlist::put_claim(user.inner().id, player1.id, true, true, &mut *connection).await;

    let submission =
        serde_json::json! {{"progress": 100, "demon": demon1, "player": "stardust1971", "video": "https://youtube.com/watch?v=1234567890"}};

    let json: serde_json::Value = clnt
        .post("/api/v1/records/", &submission)
        .expect_status(Status::Forbidden)
        .get_result()
        .await;

    assert_eq!(
        json["code"].as_i64(),
        Some(DemonlistError::NoThirdPartySubmissions.error_code() as i64)
    )
}

#[sqlx::test(migrations = "../migrations")]
async fn submit_existing_record(pool: Pool<Postgres>) {
    let (clnt, mut connection) = pointercrate_test::demonlist::setup_rocket(pool).await;

    let player1 = DatabasePlayer::by_name_or_create("stardust1971", &mut *connection).await.unwrap();
    let demon1 = pointercrate_test::demonlist::add_demon("Bloodbath", 1, 50, player1.id, player1.id, &mut *connection).await;
    let existing = pointercrate_test::demonlist::add_simple_record(70, player1.id, demon1, RecordStatus::Approved, &mut *connection).await;

    let submission = serde_json::json! {{"progress": 60, "demon": demon1, "player": "stardust1971", "video": "https://youtube.com/watch?v=1234567890", "raw_footage": "https://pointercrate.com"}};

    let json: serde_json::Value = clnt
        .post("/api/v1/records/", &submission)
        .expect_status(Status::UnprocessableEntity)
        .get_result()
        .await;

    assert_eq!(json["code"].as_i64(), Some(42217i64));
    assert_eq!(json["data"]["existing"].as_i64(), Some(existing as i64));
}

#[sqlx::test(migrations = "../migrations")]
async fn test_no_submitter_info_on_unauthed_get(pool: Pool<Postgres>) {
    let (clnt, mut connection) = pointercrate_test::demonlist::setup_rocket(pool).await;

    let player1 = DatabasePlayer::by_name_or_create("stardust1971", &mut *connection).await.unwrap();
    let demon1 = pointercrate_test::demonlist::add_demon("Bloodbath", 1, 50, player1.id, player1.id, &mut *connection).await;
    let existing = pointercrate_test::demonlist::add_simple_record(70, player1.id, demon1, RecordStatus::Approved, &mut *connection).await;

    let record: FullRecord = clnt.get(format!("/api/v1/records/{}", existing)).get_success_result().await;

    assert_eq!(record.submitter, None);
}

#[sqlx::test(migrations = "../migrations")]
async fn test_record_note_creation_and_deletion(pool: Pool<Postgres>) {
    let (clnt, mut connection) = pointercrate_test::demonlist::setup_rocket(pool).await;

    let helper = system_user_with_perms(LIST_HELPER, &mut *connection).await;
    let player1 = DatabasePlayer::by_name_or_create("stardust1971", &mut *connection).await.unwrap();
    let demon1 = pointercrate_test::demonlist::add_demon("Bloodbath", 1, 50, player1.id, player1.id, &mut *connection).await;
    let record = add_simple_record(100, player1.id, demon1, RecordStatus::Approved, &mut *connection).await;

    // Create a record note whose author is `helper`.
    let note: Note = clnt
        .post(
            format!("/api/v1/records/{}/notes", record),
            &serde_json::json! {{
                "content": "My Note",
                "is_public": false,
            }},
        )
        .authorize_as(&helper)
        .expect_status(Status::Created)
        .get_success_result()
        .await;

    // Check that the author was set correctly.
    assert_eq!(note.author.as_ref(), Some(&helper.inner().name));

    clnt.delete(format!("/api/v1/records/{}/notes/{}", record, note.id))
        .authorize_as(&helper)
        .expect_status(Status::NoContent)
        .execute()
        .await;
}

#[sqlx::test(migrations = "../migrations")]
async fn test_record_deletion_updates_player_score(pool: Pool<Postgres>) {
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

    clnt.delete(format!("/api/v1/records/{}/", record.id))
        .authorize_as(&helper)
        .header("If-Match", record.etag_string())
        .expect_status(Status::NoContent)
        .execute()
        .await;

    let player: FullPlayer = clnt
        .get(format!("/api/v1/players/{}", player.player.base.id))
        .expect_status(Status::Ok)
        .get_success_result()
        .await;

    assert_eq!(player.player.score, 0.0f64, "Deleting approved record failed to lower player score");
}
