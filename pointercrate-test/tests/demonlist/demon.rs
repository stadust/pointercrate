use pointercrate_demonlist::LIST_MODERATOR;
use rocket::http::Status;
use sqlx::{Pool, Postgres};

const DEFAULT_THUMBNAIL: &str = "https://i.ytimg.com/vi/zebrafishes/mqdefault.jpg";

#[sqlx::test(migrations = "../migrations")]
async fn test_add_demon_ratelimits(pool: Pool<Postgres>) {
    let (clnt, mut connection) = pointercrate_test::demonlist::setup_rocket(pool).await;

    let user = pointercrate_test::user::system_user_with_perms(LIST_MODERATOR, &mut connection).await;

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

#[sqlx::test(migrations = "../migrations")]
async fn test_default_thumbnail_no_video(pool: Pool<Postgres>) {
    let (clnt, mut connection) = pointercrate_test::demonlist::setup_rocket(pool).await;

    let user = pointercrate_test::user::system_user_with_perms(LIST_MODERATOR, &mut connection).await;

    let demon = serde_json::json! {{"name": "Bloodbath", "requirement": 90, "position": 1, "verifier": "Riot", "publisher": "Riot", "creators": []}};

    // first one should succeed
    let result: serde_json::Value = clnt
        .post("/api/v2/demons/", &demon)
        .authorize_as(&user)
        .expect_status(Status::Created)
        .get_result()
        .await;

    dbg!(&result);

    assert_eq!(result["data"]["thumbnail"].as_str(), Some(DEFAULT_THUMBNAIL))
}

#[sqlx::test(migrations = "../migrations")]
async fn test_default_thumbnail_linked_banned(pool: Pool<Postgres>) {
    let (clnt, mut connection) = pointercrate_test::demonlist::setup_rocket(pool).await;

    let user = pointercrate_test::user::system_user_with_perms(LIST_MODERATOR, &mut connection).await;

    let demon = serde_json::json! {{"name": "Bloodbath", "requirement": 90, "position": 1, "verifier": "Riot", "publisher": "Riot", "creators": [], "video": "https://www.youtube.com/watch?v=dQw4w9WgXcQ"}};

    sqlx::query!("INSERT INTO players (name, link_banned) VALUES ('Riot', TRUE)")
        .execute(&mut connection)
        .await
        .unwrap();

    // first one should succeed
    let result: serde_json::Value = clnt
        .post("/api/v2/demons/", &demon)
        .authorize_as(&user)
        .expect_status(Status::Created)
        .get_result()
        .await;

    dbg!(&result);

    assert_eq!(result["data"]["thumbnail"].as_str(), Some(DEFAULT_THUMBNAIL))
}

#[sqlx::test(migrations = "../migrations")]
async fn test_default_thumbnail_with_video(pool: Pool<Postgres>) {
    let (clnt, mut connection) = pointercrate_test::demonlist::setup_rocket(pool).await;

    let user = pointercrate_test::user::system_user_with_perms(LIST_MODERATOR, &mut connection).await;

    let demon = serde_json::json! {{"name": "Bloodbath", "requirement": 90, "position": 1, "verifier": "Riot", "publisher": "Riot", "creators": [], "video": "https://www.youtube.com/watch?v=dQw4w9WgXcQ"}};

    // first one should succeed
    let result: serde_json::Value = clnt
        .post("/api/v2/demons/", &demon)
        .authorize_as(&user)
        .expect_status(Status::Created)
        .get_result()
        .await;

    dbg!(&result);

    assert_eq!(
        result["data"]["thumbnail"].as_str(),
        Some("https://i.ytimg.com/vi/dQw4w9WgXcQ/mqdefault.jpg")
    )
}
