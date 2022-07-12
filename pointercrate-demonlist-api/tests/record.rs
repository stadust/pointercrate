use pointercrate_core::error::PointercrateError;
use pointercrate_demonlist::{
    error::DemonlistError,
    player::DatabasePlayer,
    record::{FullRecord, RecordStatus, Submission},
    submitter::Submitter,
};
use rocket::http::{Header, Status};
use sqlx::PgConnection;
use std::{net::IpAddr, str::FromStr};

mod setup;

#[rocket::async_test]
async fn paginate_records_unauthorized() {
    let (clnt, mut connection) = setup::setup().await;

    let (p1, r1, r2, r3) = setup_pagination_tests(&mut connection).await;

    let json: Vec<serde_json::Value> = clnt
        .get(format!("/api/v1/records/?player={}", p1))
        .expect_status(Status::Ok)
        .get_result()
        .await;

    assert_eq!(json.len(), 1);
    assert_eq!(json[0]["id"].as_i64(), Some(r1 as i64));
}

#[rocket::async_test]
async fn paginate_records_with_verified_claim() {
    let (clnt, mut connection) = setup::setup().await;

    let (p1, r1, r2, r3) = setup_pagination_tests(&mut connection).await;
    let user = setup::add_normal_user(&mut connection).await;

    setup::put_claim(user.inner().id, p1, true, false, &mut connection).await;

    let json: Vec<serde_json::Value> = clnt
        .get(format!("/api/v1/records/?player={}", p1))
        .authorize_as(&user)
        .get_result()
        .await;

    assert_eq!(json.len(), 2);
    assert_eq!(json[0]["id"].as_i64(), Some(r1 as i64));
    assert_eq!(json[1]["id"].as_i64(), Some(r2 as i64));
}

#[rocket::async_test]
async fn paginate_records_with_unverified_claim() {
    let (clnt, mut connection) = setup::setup().await;

    let (p1, r1, r2, r3) = setup_pagination_tests(&mut connection).await;
    let user = setup::add_normal_user(&mut connection).await;

    setup::put_claim(user.inner().id, p1, false, false, &mut connection).await;

    let json: Vec<serde_json::Value> = clnt
        .get(format!("/api/v1/records/?player={}", p1))
        .authorize_as(&user)
        .get_result()
        .await;

    assert_eq!(json.len(), 1);
    assert_eq!(json[0]["id"].as_i64(), Some(r1 as i64));
}

#[rocket::async_test]
async fn paginate_records_with_verified_claim_wrong_player() {
    let (clnt, mut connection) = setup::setup().await;

    let (p1, r1, r2, r3) = setup_pagination_tests(&mut connection).await;
    let user = setup::add_normal_user(&mut connection).await;

    setup::put_claim(user.inner().id, p1, true, false, &mut connection).await;

    let json: Vec<serde_json::Value> = clnt.get("/api/v1/records/?player=2").authorize_as(&user).get_result().await;

    assert_eq!(json.len(), 0);
}

async fn setup_pagination_tests(connection: &mut PgConnection) -> (i32, i32, i32, i32) {
    let player1 = DatabasePlayer::by_name_or_create("stardust1971", connection).await.unwrap();
    let player2 = DatabasePlayer::by_name_or_create("stardust1972", connection).await.unwrap();

    let demon1 = setup::add_demon("Bloodbath", 1, 87, player1.id, player1.id, connection).await;
    let demon2 = setup::add_demon("Bloodlust", 2, 53, player1.id, player1.id, connection).await;

    let r1 = setup::add_simple_record(100, player1.id, demon1, RecordStatus::Approved, connection).await;
    let r2 = setup::add_simple_record(70, player1.id, demon2, RecordStatus::Rejected, connection).await;
    let r3 = setup::add_simple_record(100, player2.id, demon2, RecordStatus::Rejected, connection).await;

    (player1.id, r1, r2, r3)
}

#[rocket::async_test]
async fn unauthed_submit_for_player_with_locked_submission() {
    let (clnt, mut connection) = setup::setup().await;

    let user = setup::add_normal_user(&mut connection).await;
    let player1 = DatabasePlayer::by_name_or_create("stardust1971", &mut connection).await.unwrap();
    let demon1 = setup::add_demon("Bloodbath", 1, 87, player1.id, player1.id, &mut connection).await;

    setup::put_claim(user.inner().id, player1.id, true, true, &mut connection).await;

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
