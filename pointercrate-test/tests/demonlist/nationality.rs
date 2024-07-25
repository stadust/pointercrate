use pointercrate_demonlist::{
    nationality::{Nationality, RankedNation},
    player::{DatabasePlayer, Player},
    LIST_MODERATOR,
};
use rocket::http::Status;
use sqlx::{Pool, Postgres};

#[sqlx::test(migrations = "../migrations")]
pub async fn test_search_nation(pool: Pool<Postgres>) {
    const PLAYER_NAME: &str = "stardust1971";
    let (client, mut connection) = pointercrate_test::demonlist::setup_rocket(pool).await;

    let player = DatabasePlayer::by_name_or_create(PLAYER_NAME, &mut connection).await.unwrap();
    let mut player = Player::by_id(player.id, &mut connection).await.unwrap();
    let nationality = Nationality {
        iso_country_code: "DE".into(),
        nation: "Germany".into(),
        subdivision: None,
    };
    player.set_nationality(Some(nationality), &mut connection).await.unwrap();

    let helper = pointercrate_test::user::system_user_with_perms(LIST_MODERATOR, &mut *connection).await;
    client.add_demon(&helper, "Bloodbath", 1, 100, PLAYER_NAME, PLAYER_NAME).await;

    let json: Vec<RankedNation> = client
        .get("/api/v1/nationalities/ranking/?name_contains=gErManY")
        .expect_status(Status::Ok)
        .get_result()
        .await;

    assert_eq!(json.len(), 1);
    assert_eq!(json[0].nationality.iso_country_code, "DE");
    assert_eq!(json[0].nationality.nation, "Germany");
}
