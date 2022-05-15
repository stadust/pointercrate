use rocket::http::{Header, Status};
use sqlx::PgConnection;

mod setup;

#[rocket::async_test]
async fn paginate_records_unauthorized() {
    let (clnt, mut connection) = setup::setup().await;

    setup::add_dummy_records(&mut connection).await;

    let json: Vec<serde_json::Value> = clnt.get("/api/v1/records/?player=1").expect_status(Status::Ok).get_result().await;

    assert_eq!(json.into_iter().map(|record| record["id"].as_i64()).collect::<Vec<_>>(), vec![
        Some(1)
    ])
}

#[rocket::async_test]
async fn paginate_records_with_verified_claim() {
    let (clnt, mut connection) = setup::setup().await;

    setup::add_dummy_records(&mut connection).await;
    let user = setup::add_normal_user(&mut connection).await;
    setup::put_claim(user.inner().id, 1, true, false, &mut connection).await;

    let json: Vec<serde_json::Value> = clnt.get("/api/v1/records/?player=1").authorize_as(&user).get_result().await;

    assert_eq!(json.into_iter().map(|record| record["id"].as_i64()).collect::<Vec<_>>(), vec![
        Some(1),
        Some(2)
    ])
}

#[rocket::async_test]
async fn paginate_records_with_unverified_claim() {
    let (clnt, mut connection) = setup::setup().await;

    setup::add_dummy_records(&mut connection).await;
    let user = setup::add_normal_user(&mut connection).await;
    setup::put_claim(user.inner().id, 1, false, false, &mut connection).await;

    let json: Vec<serde_json::Value> = clnt.get("/api/v1/records/?player=1").authorize_as(&user).get_result().await;

    assert_eq!(json.into_iter().map(|record| record["id"].as_i64()).collect::<Vec<_>>(), vec![
        Some(1)
    ])
}

#[rocket::async_test]
async fn paginate_records_with_verified_claim_wrong_player() {
    let (clnt, mut connection) = setup::setup().await;

    setup::add_dummy_records(&mut connection).await;
    let user = setup::add_normal_user(&mut connection).await;
    setup::put_claim(user.inner().id, 1, true, false, &mut connection).await;

    let json: Vec<serde_json::Value> = clnt.get("/api/v1/records/?player=2").authorize_as(&user).get_result().await;

    assert_eq!(json.into_iter().map(|record| record["id"].as_i64()).collect::<Vec<_>>(), vec![])
}
