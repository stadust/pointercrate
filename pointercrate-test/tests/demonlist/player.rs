use pointercrate_core::etag::Taggable;
use pointercrate_demonlist::{
    player::{DatabasePlayer, Player, PatchPlayer, FullPlayer},
    LIST_HELPER, nationality::{Nationality, Subdivision},
};
use pointercrate_test::TestClient;
use rocket::http::Status;
use sqlx::{PgConnection, Pool, Postgres};

async fn create_players(connection: &mut PgConnection) -> (DatabasePlayer, DatabasePlayer) {
    let mut banned = DatabasePlayer::by_name_or_create("stardust1971", &mut *connection).await.unwrap();
    banned.ban(&mut *connection).await.unwrap();
    (
        banned,
        DatabasePlayer::by_name_or_create("stardust1972", &mut *connection).await.unwrap(),
    )
}

#[sqlx::test(migrations = "../migrations")]
async fn test_unauthenticated_pagination(pool: Pool<Postgres>) {
    let (client, mut connection) = pointercrate_test::demonlist::setup_rocket(pool).await;

    let (_, unbanned) = create_players(&mut connection).await;

    let json: Vec<Player> = client.get("/api/v1/players").expect_status(Status::Ok).get_result().await;

    assert_eq!(json.len(), 1, "Pagination returned banned player");
    assert_eq!(json[0].base.id, unbanned.id);
}

#[sqlx::test(migrations = "../migrations")]
async fn test_authenticated_pagination(pool: Pool<Postgres>) {
    let (client, mut connection) = pointercrate_test::demonlist::setup_rocket(pool).await;

    let (_, unbanned) = create_players(&mut connection).await;
    let user = pointercrate_test::user::add_normal_user(&mut connection).await;

    let json: Vec<Player> = client
        .get("/api/v1/players")
        .authorize_as(&user)
        .expect_status(Status::Ok)
        .get_result()
        .await;

    assert_eq!(json.len(), 1, "Pagination returned banned player");
    assert_eq!(json[0].base.id, unbanned.id);
}

#[sqlx::test(migrations = "../migrations")]
async fn test_list_helper_pagination(pool: Pool<Postgres>) {
    let (client, mut connection) = pointercrate_test::demonlist::setup_rocket(pool).await;

    let (banned, unbanned) = create_players(&mut connection).await;
    let user = pointercrate_test::user::system_user_with_perms(LIST_HELPER, &mut connection).await;

    let json: Vec<Player> = client
        .get("/api/v1/players")
        .authorize_as(&user)
        .expect_status(Status::Ok)
        .get_result()
        .await;

    assert_eq!(json.len(), 2, "Pagination did not return banned player");
    assert_eq!(json[0].base.id, banned.id);
    assert_eq!(json[1].base.id, unbanned.id);
}



#[sqlx::test(migrations = "../migrations")]
async fn test_patch_player_nationality(pool: Pool<Postgres>) {
    let (client, mut connection) = pointercrate_test::demonlist::setup_rocket(pool).await;
    let player = DatabasePlayer::by_name_or_create("stardust1971", &mut *connection).await.unwrap();
    let user = pointercrate_test::user::system_user_with_perms(LIST_HELPER, &mut connection).await;

    let etag = Player::by_id(player.id, &mut *connection).await.unwrap().upgrade(&mut *connection).await.unwrap().etag_string();

    let json: FullPlayer = client.patch(format!("/api/v1/players/{}/", player.id), &serde_json::json!({"nationality": "United Kingdom", "subdivision": "ENG"}))
        .authorize_as(&user)
        .header("If-Match", etag)
        .expect_status(Status::Ok)
        .get_success_result()
        .await;

    assert_eq!(json.player.nationality, Some(Nationality {iso_country_code: "GB".into(), nation: "United Kingdom".into(), subdivision: Some(Subdivision {iso_code: "ENG".into(), name: "England".into()})}));
}