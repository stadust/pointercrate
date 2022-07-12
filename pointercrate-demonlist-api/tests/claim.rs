use pointercrate_demonlist::player::{claim::PlayerClaim, DatabasePlayer};
use rocket::http::Status;

mod setup;

#[rocket::async_test]
async fn test_put_claim() {
    let (client, mut connection) = setup::setup().await;
    let user = setup::add_normal_user(&mut connection).await;

    let player_id = DatabasePlayer::by_name_or_create("stardust1971", &mut connection).await.unwrap().id;

    let json: PlayerClaim = client
        .put(format!("/api/v1/players/{}/claims/", player_id))
        .authorize_as(&user)
        .expect_status(Status::Created)
        .expect_header(
            "Location",
            format!("/api/v1/players/{}/claims/{}/", player_id, user.inner().id).as_str(),
        )
        .get_result()
        .await;

    assert_eq!(json, PlayerClaim {
        user_id: user.inner().id,
        player_id,
        verified: false,
        lock_submissions: false
    });
}
