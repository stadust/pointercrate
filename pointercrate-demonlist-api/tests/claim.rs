use pointercrate_demonlist::player::claim::PlayerClaim;
use rocket::http::Status;

mod setup;

#[rocket::async_test]
async fn test_put_claim() {
    let (client, mut connection) = setup::setup().await;
    let user = setup::add_normal_user(&mut connection).await;

    let json: PlayerClaim = client
        .put("/api/v1/players/1/claims/")
        .authorize_as(&user)
        .expect_status(Status::Created)
        .expect_header("Location", format!("/api/v1/players/1/claims/{}/", user.inner().id).as_str())
        .get_result()
        .await;

    assert_eq!(json, PlayerClaim {
        user_id: user.inner().id,
        player_id: 1,
        verified: false,
        lock_submissions: false
    });
}
