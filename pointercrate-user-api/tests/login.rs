use rocket::http::{ContentType, Header, Status};

mod setup;

#[rocket::async_test]
pub async fn test_login() {
    let (client, user, _) = setup::setup_with_admin_user().await;

    let response = client
        .post("/api/v1/auth/")
        .header(Header::new("Authorization", "Basic UGF0cmljazpiYWQgcGFzc3dvcmQ="))
        .header(Header::new("X-Real-IP", "127.0.0.1"))
        .dispatch()
        .await;

    assert_eq!(response.status(), Status::Ok);

    let body_text = response.into_string().await.unwrap();
    let json: serde_json::Value = serde_json::from_str(&body_text).unwrap();

    assert_eq!(user.inner().id as i64, json["data"]["id"].as_i64().unwrap());
    assert!(user.validate_access_token(json["token"].as_str().unwrap()).is_ok());
}

#[rocket::async_test]
pub async fn test_login_malformed_auth_header() {
    let (client, ..) = setup::setup_with_admin_user().await;

    let response = client
        .post("/api/v1/auth/")
        .header(Header::new("Authorization", "Basic öüßd"))
        .header(Header::new("X-Real-IP", "127.0.0.1"))
        .dispatch()
        .await;

    assert_eq!(response.status(), Status::BadRequest)
}

#[rocket::async_test]
pub async fn test_login_wrong_password() {
    let (client, ..) = setup::setup_with_admin_user().await;

    let response = client
        .post("/api/v1/auth/")
        .header(Header::new("Authorization", "Basic UGF0cmljazp3cm9uZyBwYXNzd29yZA=="))
        .header(Header::new("X-Real-IP", "127.0.0.1"))
        .dispatch()
        .await;

    assert_eq!(response.status(), Status::Unauthorized)
}

#[rocket::async_test]
pub async fn test_login_wrong_username() {
    let (client, ..) = setup::setup_with_admin_user().await;

    let response = client
        .post("/api/v1/auth/")
        .header(Header::new("Authorization", "Basic UGF0cmlja2dmZHM6d3JvbmcgcGFzc3dvcmQ="))
        .header(Header::new("X-Real-IP", "127.0.0.1"))
        .dispatch()
        .await;

    assert_eq!(response.status(), Status::Unauthorized)
}

#[rocket::async_test]
pub async fn test_login_no_header() {
    let (client, ..) = setup::setup_with_admin_user().await;

    let response = client
        .post("/api/v1/auth/")
        .header(Header::new("X-Real-IP", "127.0.0.1"))
        .dispatch()
        .await;

    assert_eq!(response.status(), Status::NotFound)
}
