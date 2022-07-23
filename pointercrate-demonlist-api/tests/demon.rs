use pointercrate_demonlist::LIST_MODERATOR;
use rocket::http::Status;

mod setup;

#[rocket::async_test]
async fn test_add_demon_ratelimits() {
    let (clnt, mut connection) = setup::setup().await;

    let user = setup::system_user_with_perms(LIST_MODERATOR, &mut connection).await;

    let demon = serde_json::json! {{"name": "Bloodbath", "requirement": 90, "position": 1, "verifier": "Riot", "publisher": "Riot", "creators": []}};

    // first one should succeed
    clnt.post("/api/v2/demons/", &demon)
        .authorize_as(&user)
        .expect_status(Status::Created)
        .execute()
        .await;

    // second one should hit the "1 per minute" ratelimit
    let result: serde_json::Value = clnt
        .post("/api/v2/demons/", &demon)
        .authorize_as(&user)
        .expect_status(Status::TooManyRequests)
        .get_result()
        .await;

    assert_eq!(result["code"].as_i64(), Some(42900))
}
