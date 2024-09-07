use pointercrate_user::ADMINISTRATOR;
use rocket::http::Status;
use sqlx::{Pool, Postgres};

#[sqlx::test(migrations = "../migrations")]
pub async fn test_login_with_ratelimit(pool: Pool<Postgres>) {
    let (client, mut connection) = pointercrate_test::user::setup_rocket(pool).await;

    let mut user = pointercrate_test::user::system_user_with_perms(ADMINISTRATOR, &mut *connection).await;

    for _ in 0..3 {
        let response: serde_json::Value = client
            .post("/api/v1/auth/", &())
            .header("Authorization", "Basic UGF0cmljazpiYWQgcGFzc3dvcmQ=")
            .header("X-Real-IP", "127.0.0.1")
            .expect_status(Status::Ok)
            .get_result()
            .await;

        assert_eq!(user.user().id as i64, response["data"]["id"].as_i64().unwrap());

        // validate_access_token takes ownership, but it gives back the object if verification is successfuly
        user = user.validate_access_token(response["token"].as_str().unwrap()).unwrap();
    }

    // After 3 requests, both valid and invalid requests should just return a 429 response
    client
        .post("/api/v1/auth/", &())
        .header("Authorization", "Basic UGF0cmljazpiYWQgcGFzc3dvcmQ=")
        .header("X-Real-IP", "127.0.0.1")
        .expect_status(Status::TooManyRequests)
        .execute()
        .await;

    client
        .post("/api/v1/auth/", &())
        .header("Authorization", "Basic kjföldsa")
        .header("X-Real-IP", "127.0.0.1")
        .expect_status(Status::TooManyRequests)
        .execute()
        .await;
}

#[sqlx::test(migrations = "../migrations")]
pub async fn test_login_malformed_auth_header(pool: Pool<Postgres>) {
    let (client, _) = pointercrate_test::user::setup_rocket(pool).await;

    client
        .post("/api/v1/auth/", &())
        .header("Authorization", "Basic öüßd")
        .header("X-Real-IP", "127.0.0.1")
        .expect_status(Status::BadRequest)
        .execute()
        .await;
}

#[sqlx::test(migrations = "../migrations")]
pub async fn test_login_wrong_password(pool: Pool<Postgres>) {
    let (client, mut connection) = pointercrate_test::user::setup_rocket(pool).await;

    // Make sure the user we're trying to log in to exists
    let _ = pointercrate_test::user::system_user_with_perms(ADMINISTRATOR, &mut *connection).await;

    client
        .post("/api/v1/auth/", &())
        .header("Authorization", "Basic UGF0cmljazp3cm9uZyBwYXNzd29yZA==")
        .header("X-Real-IP", "127.0.0.1")
        .expect_status(Status::Unauthorized)
        .execute()
        .await;
}

#[sqlx::test(migrations = "../migrations")]
pub async fn test_login_wrong_username(pool: Pool<Postgres>) {
    let (client, _) = pointercrate_test::user::setup_rocket(pool).await;

    client
        .post("/api/v1/auth/", &())
        .header("Authorization", "Basic UGF0cmlja2dmZHM6d3JvbmcgcGFzc3dvcmQ=")
        .header("X-Real-IP", "127.0.0.1")
        .expect_status(Status::Unauthorized)
        .execute()
        .await;
}

#[sqlx::test(migrations = "../migrations")]
pub async fn test_login_no_header(pool: Pool<Postgres>) {
    let (client, _) = pointercrate_test::user::setup_rocket(pool).await;

    client
        .post("/api/v1/auth/", &())
        .header("X-Real-IP", "127.0.0.1")
        .expect_status(Status::NotFound)
        .execute()
        .await;
}
