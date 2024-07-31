use pointercrate_core::pagination::PaginationParameters;
use pointercrate_core_api::pagination::LinksBuilder;
use pointercrate_demonlist::{
    demon::{Demon, DemonPositionPagination},
    player::DatabasePlayer,
    LIST_MODERATOR,
};
use rocket::http::Status;
use sqlx::{Pool, Postgres};

#[sqlx::test(migrations = "../migrations")]
async fn test_add_demon_ratelimits(pool: Pool<Postgres>) {
    let (clnt, mut connection) = pointercrate_test::demonlist::setup_rocket(pool).await;

    let user = pointercrate_test::user::system_user_with_perms(LIST_MODERATOR, &mut *connection).await;

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
async fn test_demon_pagination(pool: Pool<Postgres>) {
    /// The URL of the endpoint we are testing
    const URL: &str = "/api/v2/demons/listed/";

    let (clnt, mut connection) = pointercrate_test::demonlist::setup_rocket(pool).await;

    let player = DatabasePlayer::by_name_or_create("stardust1971", &mut *connection).await.unwrap();

    // Pagination on an empty table results in an empty response with empty links header.
    //
    // Regression test for #77
    let (demons, links) = clnt.get(URL).get_pagination_result::<Demon>().await;

    assert!(demons.is_empty(), "{:?}", demons);
    assert_eq!(links, LinksBuilder::new(URL).generate(&DemonPositionPagination::default()).unwrap());

    // Let's add some data to the database and do actual tests!
    let id1 = pointercrate_test::demonlist::add_demon("Bloodbath", 1, 100, player.id, player.id, &mut *connection).await;
    let id2 = pointercrate_test::demonlist::add_demon("Bloodbath 2", 2, 100, player.id, player.id, &mut *connection).await;
    let id3 = pointercrate_test::demonlist::add_demon("Bloodbath 3", 3, 100, player.id, player.id, &mut *connection).await;

    // Normal pagination: Get the demon at position 2 via after=1 and limit=1. We should get both "next" and "previous" pages
    // for the first and third demons! Let's throw in a requirement=100 as well, to test that these parameters do indeed get propagated into the Links headers
    let base = DemonPositionPagination {
        requirement: Some(100),
        params: PaginationParameters {
            limit: 1,
            after: Some(1),
            ..Default::default()
        },
        ..Default::default()
    };
    let (demons, links) = clnt
        .get(format!("{}?{}", URL, serde_urlencoded::to_string(&base).unwrap()))
        .get_pagination_result::<Demon>()
        .await;

    assert_eq!(demons.len(), 1);
    assert_eq!(demons[0].base.id, id2);

    let expected = LinksBuilder::new(URL).with_first(0).with_last(4).with_next(2).with_previous(2);
    assert_eq!(links, expected.generate(&base).unwrap());

    // The same, but in reverse Get the demon at position 2 via before=3 and limit=1. We should get both "next" and "previous" pages
    // for the first and third demons! Let's throw in a requirement=100 as well, to test that these parameters do indeed get propagated into the Links headers
    let base = DemonPositionPagination {
        requirement: Some(100),
        params: PaginationParameters {
            limit: 1,
            before: Some(3),
            ..Default::default()
        },
        ..Default::default()
    };
    let (demons, links) = clnt
        .get(format!("{}?{}", URL, serde_urlencoded::to_string(&base).unwrap()))
        .get_pagination_result::<Demon>()
        .await;

    assert_eq!(demons.len(), 1);
    assert_eq!(demons[0].base.id, id2);

    let expected = LinksBuilder::new(URL).with_first(0).with_last(4).with_next(2).with_previous(2);
    assert_eq!(links, expected.generate(&base).unwrap());

    // Query an empty page by only setting before=1. We should still get a "next" link, with after=0 (e.g. before minus one),
    // but no "prev" link
    let base = DemonPositionPagination {
        params: PaginationParameters {
            before: Some(1),
            ..Default::default()
        },
        ..Default::default()
    };
    let (demons, links) = clnt
        .get(format!("{}?{}", URL, serde_urlencoded::to_string(&base).unwrap()))
        .get_pagination_result::<Demon>()
        .await;

    assert_eq!(demons.len(), 0);

    let expected = LinksBuilder::new(URL).with_first(0).with_last(4).with_next(0);

    assert_eq!(links, expected.generate(&base).unwrap());

    // Query an empty page by setting "before" and "after" to an empty range. Should result in a response with only "first" and "last" headers set
    let base = DemonPositionPagination {
        params: PaginationParameters {
            before: Some(2),
            after: Some(1),
            ..Default::default()
        },
        ..Default::default()
    };
    let (demons, links) = clnt
        .get(format!("{}?{}", URL, serde_urlencoded::to_string(&base).unwrap()))
        .get_pagination_result::<Demon>()
        .await;

    assert_eq!(demons.len(), 0);

    let expected = LinksBuilder::new(URL).with_first(0).with_last(4);

    assert_eq!(links, expected.generate(&base).unwrap());

    // Query with limit=3, which should result in all three demons being returned, and only "first" and "last" headers set (since there are no other pages)
    let base = DemonPositionPagination::default();
    let (demons, links) = clnt
        .get(format!("{}?{}", URL, serde_urlencoded::to_string(&base).unwrap()))
        .get_pagination_result::<Demon>()
        .await;

    assert_eq!(demons.len(), 3);
    assert_eq!(demons[0].base.id, id1);
    assert_eq!(demons[1].base.id, id2);
    assert_eq!(demons[2].base.id, id3);

    let expected = LinksBuilder::new(URL).with_first(0).with_last(4);

    assert_eq!(links, expected.generate(&base).unwrap());

    // Query with limit=2 and before=4, to test that we still return the results in ascending order
    let base = DemonPositionPagination {
        params: PaginationParameters {
            before: Some(4),
            limit: 2,
            ..Default::default()
        },
        ..Default::default()
    };
    let (demons, links) = clnt
        .get(format!("{}?{}", URL, serde_urlencoded::to_string(&base).unwrap()))
        .get_pagination_result::<Demon>()
        .await;

    assert_eq!(demons.len(), 2);
    assert_eq!(demons[0].base.id, id2);
    assert_eq!(demons[1].base.id, id3);

    let expected = LinksBuilder::new(URL)
        .with_first(0)
        .with_last(4)
        .with_next(3) // FIXME: This `next` link should not have been returned (currently we always return a "next" link if a `before` parameter is set though)
        .with_previous(2);

    assert_eq!(links, expected.generate(&base).unwrap());
}
