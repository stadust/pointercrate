use pointercrate_user::auth::legacy::Registration;
use rocket::http::Status;
use sqlx::{Pool, Postgres};

#[sqlx::test(migrations = "../migrations")]
pub async fn register_new(pool: Pool<Postgres>) {
    let (client, _) = pointercrate_test::user::setup_rocket(pool).await;

    client
        .post(
            "/api/v1/auth/register/",
            &Registration {
                name: "Patrick".to_string(),
                password: "bad password".to_string(),
            },
        )
        .header("X-Real-IP", "127.0.0.1")
        .expect_status(Status::Created)
        .execute()
        .await;
}

#[sqlx::test(migrations = "../migrations")]
pub async fn register_taken_username(pool: Pool<Postgres>) {
    let (client, mut connection) = pointercrate_test::user::setup_rocket(pool).await;

    let _ = pointercrate_test::user::add_normal_user(&mut *connection).await;

    let _response = client
        .post(
            "/api/v1/auth/register/",
            &Registration {
                name: "Patrick".to_string(),
                password: "bad password".to_string(),
            },
        )
        .header("X-Real-IP", "127.0.0.1")
        .expect_status(Status::Conflict)
        .execute()
        .await;
}
