use pointercrate_demonlist::player::claim::PlayerClaim;
use rocket::http::{Header, Status};

mod setup;

#[rocket::async_test]
async fn test_put_claim() {
    let (client, mut connection) = setup::setup().await;
    let user = setup::add_normal_user(&mut connection).await;

    let response = client
        .put("/api/v1/players/1/claims/")
        .header(Header::new("Authorization", format!("Bearer {}", user.generate_access_token())))
        .header(Header::new("X-Real-IP", "127.0.0.1"))
        .dispatch()
        .await;

    assert_eq!(response.status(), Status::Created);
    assert_eq!(
        response.headers().get_one("Location"),
        Some(format!("/api/v1/players/1/claims/{}/", user.inner().id).as_str())
    );

    let body_text = response.into_string().await.unwrap();
    let json: PlayerClaim = serde_json::from_str(&body_text).unwrap();

    assert_eq!(json, PlayerClaim {
        user_id: user.inner().id,
        player_id: 1,
        verified: false,
        lock_submissions: false
    });
}
