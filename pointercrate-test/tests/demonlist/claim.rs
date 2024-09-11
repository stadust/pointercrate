use pointercrate_demonlist::player::{claim::PlayerClaim, DatabasePlayer};
use rocket::http::Status;
use sqlx::{Pool, Postgres};

#[sqlx::test(migrations = "../migrations")]
async fn test_put_claim(pool: Pool<Postgres>) {
    let (client, mut connection) = pointercrate_test::demonlist::setup_rocket(pool).await;
    let user = pointercrate_test::user::add_normal_user(&mut *connection).await;

    let player_id = DatabasePlayer::by_name_or_create("stardust1971", &mut *connection)
        .await
        .unwrap()
        .id;

    let json: PlayerClaim = client
        .put(format!("/api/v1/players/{}/claims/", player_id))
        .authorize_as(&user)
        .expect_status(Status::Created)
        .expect_header(
            "Location",
            format!("/api/v1/players/{}/claims/{}/", player_id, user.user().id).as_str(),
        )
        .get_result()
        .await;

    assert_eq!(
        json,
        PlayerClaim {
            user_id: user.user().id,
            player_id,
            verified: false,
            lock_submissions: false
        }
    );
}
