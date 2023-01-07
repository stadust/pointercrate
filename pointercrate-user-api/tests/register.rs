use pointercrate_user::Registration;
use rocket::http::{ContentType, Header, Status};
use sqlx::{pool::PoolConnection, Pool, Postgres};

mod setup;

#[sqlx::test(migrations = "../migrations")]
pub async fn register_new(pool: Pool<Postgres>) {
    let client = setup::setup(pool).await;

    let response = client
        .post("/api/v1/auth/register/")
        .header(ContentType::JSON)
        .json(&Registration {
            name: "Patrick".to_string(),
            password: "bad password".to_string(),
        })
        .header(Header::new("X-Real-IP", "127.0.0.1"))
        .dispatch()
        .await;

    assert_eq!(response.status(), Status::Created)
}

#[sqlx::test(migrations = "../migrations")]
pub async fn register_taken_username(pool: Pool<Postgres>) {
    let (client, _) = setup::setup_with_admin_user(pool).await;

    let response = client
        .post("/api/v1/auth/register/")
        .header(ContentType::JSON)
        .json(&Registration {
            name: "Patrick".to_string(),
            password: "bad password".to_string(),
        })
        .header(Header::new("X-Real-IP", "127.0.0.1"))
        .dispatch()
        .await;

    assert_eq!(response.status(), Status::Conflict)
}
