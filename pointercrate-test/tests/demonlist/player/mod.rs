use pointercrate_demonlist::record::RecordStatus;
use pointercrate_demonlist::{
    nationality::{Nationality, Subdivision},
    player::{DatabasePlayer, FullPlayer, Player},
    LIST_HELPER, LIST_MODERATOR,
};
use rocket::http::Status;
use serde_json::json;
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
    let player = DatabasePlayer::by_name_or_create("stardust1971", &mut connection).await.unwrap();
    let user = pointercrate_test::user::system_user_with_perms(LIST_HELPER, &mut connection).await;

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

#[sqlx::test(migrations = "../migrations")]
async fn test_me(pool: Pool<Postgres>) {
    let (client, mut connection) = pointercrate_test::demonlist::setup_rocket(pool).await;

    // Assert 401 without authentication
    client.get("/api/v1/players/me").expect_status(Status::Unauthorized).execute().await;

    let authenticated_user = pointercrate_test::user::add_normal_user(&mut connection).await;
    let user = authenticated_user.user();

    // Assert 404 when authorized, but claim doesn't exist
    client
        .get("/api/v1/players/me")
        .authorize_as(&authenticated_user)
        .expect_status(Status::NotFound)
        .execute()
        .await;

    // Create claim
    let player = DatabasePlayer::by_name_or_create("stardust1971", &mut connection).await.unwrap();
    player
        .initiate_claim(user.id, &mut connection)
        .await
        .unwrap()
        .set_verified(true, &mut connection)
        .await
        .unwrap();
    let player = Player::by_id(player.id, &mut connection)
        .await
        .unwrap()
        .upgrade(&mut connection)
        .await
        .unwrap();

    // Authorized and claim exists
    assert_eq!(
        client
            .get("/api/v1/players/me")
            .authorize_as(&authenticated_user)
            .expect_status(Status::Ok)
            .get_success_result::<FullPlayer>()
            .await,
        player
    );
}

#[sqlx::test(migrations = "../migrations")]
async fn test_players_pagination(pool: Pool<Postgres>) {
    let (client, mut connection) = pointercrate_test::demonlist::setup_rocket(pool).await;
    let moderator = pointercrate_test::user::system_user_with_perms(LIST_MODERATOR, &mut connection).await;

    // create players
    let _ = DatabasePlayer::by_name_or_create("stardust19701", &mut connection).await.unwrap(); // no nationality, no subdivision
    let player2 = DatabasePlayer::by_name_or_create("stardust19702", &mut connection).await.unwrap(); // has nationality, no subdivision
    let player3 = DatabasePlayer::by_name_or_create("stardust19703", &mut connection).await.unwrap(); // has nationality, has subdivision

    client
        .patch_player(player2.id, &moderator, serde_json::json!({"nationality": "GB"}))
        .await
        .execute()
        .await;

    client
        .patch_player(
            player3.id,
            &moderator,
            serde_json::json!({"nationality": "GB", "subdivision": "ENG"}),
        )
        .await
        .execute()
        .await;

    // test if all players are returned by the endpoint (without filters)
    let players: Vec<Player> = client.get("/api/v1/players").expect_status(Status::Ok).get_result().await;

    assert_eq!(players.len(), 3, "Not all players are listed");

    assert_eq!(players[0].nationality, None);
    assert_eq!(
        players[1].nationality,
        Some(Nationality {
            iso_country_code: "GB".into(),
            nation: "United Kingdom".into(),
            subdivision: None,
        })
    );
    assert_eq!(
        players[2].nationality,
        Some(Nationality {
            iso_country_code: "GB".into(),
            nation: "United Kingdom".into(),
            subdivision: Some(Subdivision {
                iso_code: "ENG".into(),
                name: "England".into(),
            }),
        })
    );

    // test subdivision filter
    let subdivision_filtered_players: Vec<Player> = client
        .get("/api/v1/players?subdivision=ENG")
        .expect_status(Status::Ok)
        .get_result()
        .await;

    assert_eq!(
        subdivision_filtered_players.len(),
        1,
        "Subdivision filter did not return the correct number of players"
    );

    assert_eq!(subdivision_filtered_players[0].base.id, player3.id);
    assert_eq!(
        subdivision_filtered_players[0].nationality,
        Some(Nationality {
            iso_country_code: "GB".into(),
            nation: "United Kingdom".into(),
            subdivision: Some(Subdivision {
                iso_code: "ENG".into(),
                name: "England".into(),
            }),
        })
    );

    // test nation filter
    let nation_filtered_players: Vec<Player> = client.get("/api/v1/players?nation=GB").expect_status(Status::Ok).get_result().await;

    assert_eq!(
        nation_filtered_players.len(),
        2,
        "Nation filter did not return the correct number of players"
    );

    assert_eq!(nation_filtered_players[0].base.id, player2.id);
    assert_eq!(nation_filtered_players[1].base.id, player3.id);

    assert_eq!(
        nation_filtered_players[0].nationality,
        Some(Nationality {
            iso_country_code: "GB".into(),
            nation: "United Kingdom".into(),
            subdivision: None,
        })
    );
    assert_eq!(
        nation_filtered_players[1].nationality,
        Some(Nationality {
            iso_country_code: "GB".into(),
            nation: "United Kingdom".into(),
            subdivision: Some(Subdivision {
                iso_code: "ENG".into(),
                name: "England".into(),
            }),
        })
    );
}

#[sqlx::test(migrations = "../migrations")]
async fn test_player_merge(pool: Pool<Postgres>) {
    let (client, mut connection) = pointercrate_test::demonlist::setup_rocket(pool).await;
    let moderator = pointercrate_test::user::system_user_with_perms(LIST_MODERATOR, &mut connection).await;

    /*
     * We're creating two players with approved records on the same demon (but different progress) and then rename them to have the same name
     * This should merge the two records (keeping the higher progress) and delete one of the player objects.
     */

    let player1 = DatabasePlayer::by_name_or_create("stardust1971", &mut connection).await.unwrap();
    let player2 = DatabasePlayer::by_name_or_create("stardust1972", &mut connection).await.unwrap();

    let demon1 = pointercrate_test::demonlist::add_demon("Bloodbath", 1, 87, player1.id, player1.id, &mut connection).await;

    pointercrate_test::demonlist::add_simple_record(90, player1.id, demon1, RecordStatus::Approved, &mut connection).await;
    pointercrate_test::demonlist::add_simple_record(95, player2.id, demon1, RecordStatus::Approved, &mut connection).await;

    let patched: FullPlayer = client
        .patch_player(player2.id, &moderator, json! {{"name": "stardust1971"}})
        .await
        .get_success_result()
        .await;

    assert_eq!(patched.records.len(), 1);
    assert_eq!(patched.records[0].progress, 95);
    assert_eq!(patched.player.base.id, player2.id);

    client
        .get(&format!("/api/v1/players/{}/", player1.id))
        .expect_status(Status::NotFound)
        .execute()
        .await;
}
