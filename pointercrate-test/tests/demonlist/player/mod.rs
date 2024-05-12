use pointercrate_demonlist::{
    nationality::{Nationality, Subdivision},
    player::{DatabasePlayer, FullPlayer, Player},
    LIST_HELPER,
};
use rocket::http::Status;
use sqlx::{PgConnection, Pool, Postgres};

mod score;

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

    let (_, unbanned) = create_players(&mut *connection).await;

    let json: Vec<Player> = client.get("/api/v1/players").expect_status(Status::Ok).get_result().await;

    assert_eq!(json.len(), 1, "Pagination returned banned player");
    assert_eq!(json[0].base.id, unbanned.id);
}

#[sqlx::test(migrations = "../migrations")]
async fn test_authenticated_pagination(pool: Pool<Postgres>) {
    let (client, mut connection) = pointercrate_test::demonlist::setup_rocket(pool).await;

    let (_, unbanned) = create_players(&mut *connection).await;
    let user = pointercrate_test::user::add_normal_user(&mut *connection).await;

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

    let (banned, unbanned) = create_players(&mut *connection).await;
    let user = pointercrate_test::user::system_user_with_perms(LIST_HELPER, &mut *connection).await;

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
    let user = pointercrate_test::user::system_user_with_perms(LIST_HELPER, &mut *connection).await;

    // Try to set subdivision when no nation is set. Should fail.
    let result: serde_json::Value = client
        .patch_player(player.id, &user, serde_json::json!({"subdivision": "ENG"}))
        .await
        .expect_status(Status::Conflict)
        .get_result()
        .await;

    assert_eq!(result["code"], 40907);

    // Patch both nationality and subdivision
    let patched_player: FullPlayer = client
        .patch_player(
            player.id,
            &user,
            serde_json::json!({"nationality": "United Kingdom", "subdivision": "ENG"}),
        )
        .await
        .get_success_result()
        .await;

    assert_eq!(
        patched_player.player.nationality,
        Some(Nationality {
            iso_country_code: "GB".into(),
            nation: "United Kingdom".into(),
            subdivision: Some(Subdivision {
                iso_code: "ENG".into(),
                name: "England".into()
            })
        })
    );

    // Patch only subdivision, nationality should remain untouched
    let patched_player: FullPlayer = client
        .patch_player(
            player.id,
            &user,
            serde_json::json!({"nationality": "United Kingdom", "subdivision": "SCT"}),
        )
        .await
        .get_success_result()
        .await;

    assert_eq!(
        patched_player.player.nationality,
        Some(Nationality {
            iso_country_code: "GB".into(),
            nation: "United Kingdom".into(),
            subdivision: Some(Subdivision {
                iso_code: "SCT".into(),
                name: "Scotland".into()
            })
        })
    );

    // Patch nation, but to the one we already have. Shouldn't change anything.
    client
        .patch_player(player.id, &user, serde_json::json!({"nationality": "United Kingdom"}))
        .await
        .expect_status(Status::NotModified)
        .execute()
        .await;

    // Patch only nationality. Should reset subdivision
    let patched_player: FullPlayer = client
        .patch_player(player.id, &user, serde_json::json!({"nationality": "Germany"}))
        .await
        .get_success_result()
        .await;

    assert_eq!(
        patched_player.player.nationality,
        Some(Nationality {
            iso_country_code: "DE".into(),
            nation: "Germany".into(),
            subdivision: None
        })
    );

    // Nonsense nationality/subdivision combo should be rejected
    let result: serde_json::Value = client
        .patch_player(
            player.id,
            &user,
            serde_json::json!({"nationality": "Belgium", "subdivision": "ENG"}),
        )
        .await
        .expect_status(Status::NotFound)
        .get_result()
        .await;

    assert_eq!(result["code"], 40401);
    assert_eq!(result["data"]["nation_code"], "BE");
    assert_eq!(result["data"]["subdivision_code"], "ENG");
}
