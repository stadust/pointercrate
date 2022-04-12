use rocket::http::{Header, Status};
use sqlx::PgConnection;

mod setup;

#[rocket::async_test]
async fn paginate_records_unauthorized() {
    let (clnt, mut connection) = setup::setup().await;

    setup::add_dummy_records(&mut connection).await;

    let response = clnt
        .get("/api/v1/records/?player=1")
        .header(Header::new("X-Real-IP", "127.0.0.1"))
        .dispatch()
        .await;

    assert_eq!(response.status(), Status::Ok);

    let body_text = response.into_string().await.unwrap();
    let json: Vec<serde_json::Value> = serde_json::from_str(&body_text).unwrap();

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

    let response = clnt
        .get("/api/v1/records/?player=1")
        .header(Header::new("Authorization", format!("Bearer {}", user.generate_access_token())))
        .header(Header::new("X-Real-IP", "127.0.0.1"))
        .dispatch()
        .await;

    assert_eq!(response.status(), Status::Ok);

    let body_text = response.into_string().await.unwrap();
    let json: Vec<serde_json::Value> = serde_json::from_str(&body_text).unwrap();

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

    let response = clnt
        .get("/api/v1/records/?player=1")
        .header(Header::new("Authorization", format!("Bearer {}", user.generate_access_token())))
        .header(Header::new("X-Real-IP", "127.0.0.1"))
        .dispatch()
        .await;

    assert_eq!(response.status(), Status::Ok);

    let body_text = response.into_string().await.unwrap();
    let json: Vec<serde_json::Value> = serde_json::from_str(&body_text).unwrap();

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

    let response = clnt
        .get("/api/v1/records/?player=2")
        .header(Header::new("Authorization", format!("Bearer {}", user.generate_access_token())))
        .header(Header::new("X-Real-IP", "127.0.0.1"))
        .dispatch()
        .await;

    assert_eq!(response.status(), Status::Ok);

    let body_text = response.into_string().await.unwrap();
    let json: Vec<serde_json::Value> = serde_json::from_str(&body_text).unwrap();

    assert_eq!(json.into_iter().map(|record| record["id"].as_i64()).collect::<Vec<_>>(), vec![])
}
