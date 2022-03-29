use pointercrate_user::Registration;
use rocket::http::{ContentType, Header, Status};

mod setup;

#[rocket::async_test]
#[serial_test::serial]
pub async fn register_new() {
    let (client, _) = setup::setup().await;

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

#[rocket::async_test]
#[serial_test::serial]
pub async fn register_taken_username() {
    let (client, ..) = setup::setup_with_admin_user().await;

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
