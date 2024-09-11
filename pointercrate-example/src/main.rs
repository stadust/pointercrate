use maud::html;
use pointercrate_core::error::CoreError;
use pointercrate_core::pool::PointercratePool;
use pointercrate_core_api::{error::ErrorResponder, maintenance::MaintenanceFairing};
use pointercrate_core_pages::{
    footer::{Footer, FooterColumn, Link},
    navigation::{NavigationBar, TopLevelNavigationBarItem},
    PageConfiguration,
};
use pointercrate_demonlist::LIST_ADMINISTRATOR;
use pointercrate_demonlist_pages::account::{
    demons::DemonsTab, list_integration::ListIntegrationTab, players::PlayersPage, records::RecordsPage,
};
use pointercrate_user::MODERATOR;
use pointercrate_user_pages::account::{profile::ProfileTab, users::UsersTab, AccountPageConfig};
use rocket::{build, catch, fs::FileServer, get, response::Redirect, uri, Rocket};
use shuttle_runtime::SecretStore;

#[catch(404)]
fn catch_404() -> ErrorResponder {
    CoreError::NotFound.into()
}

#[rocket::catch(422)]
fn catch_422() -> ErrorResponder {
    CoreError::UnprocessableEntity.into()
}

#[get("/")]
fn home() -> Redirect {
    Redirect::to(uri!("/list/"))
}

async fn configure_rocket(secrets: &SecretStore) -> Result<Rocket<rocket::Build>, Box<dyn std::error::Error>> {
    let pool = PointercratePool::init(secrets).await;

    let rocket = build()
        .manage(pool)
        .manage(page_configuration())
        .register("/", rocket::catchers![catch_404, catch_422])
        .mount("/", rocket::routes![home]);

    let mut permissions_manager = pointercrate_user::default_permissions_manager();
    permissions_manager.merge_with(pointercrate_demonlist::default_permissions_manager());

    let rocket = rocket.manage(permissions_manager);

    let account_page_config = AccountPageConfig::default()
        .with_page(ProfileTab)
        .with_page(ListIntegrationTab("https://discord.com/invite/W7Eqqj8NG2"))
        .with_page(UsersTab(vec![MODERATOR, LIST_ADMINISTRATOR]))
        .with_page(DemonsTab)
        .with_page(PlayersPage)
        .with_page(RecordsPage);

    let rocket = rocket.manage(account_page_config);
    let rocket = rocket.attach(MaintenanceFairing::new(false));
    let rocket = pointercrate_demonlist_api::setup(rocket);
    let rocket = pointercrate_user_api::setup(rocket);

    Ok(rocket
        .mount("/static/core", FileServer::from("pointercrate-core-pages/static"))
        .mount("/static/demonlist", FileServer::from("pointercrate-demonlist-pages/static"))
        .mount("/static/user", FileServer::from("pointercrate-user-pages/static")))
}

fn page_configuration() -> PageConfiguration {
    let nav_bar = NavigationBar::new("/static/core/thecscl.png")
        .with_item(
            TopLevelNavigationBarItem::new(
                "/list/",
                html! {
                    span {
                        "Challenge List"
                    }
                },
            )
            .with_sub_item("/list/statsviewer/", html! {"Stats Viewer"})
            .with_sub_item("/list/?submitter=true", html! {"Record Submitter"})
            .with_sub_item("/list/?timemachine=true", html! {"Time Machine"}),
        )
        .with_item(TopLevelNavigationBarItem::new(
            "/login/",
            html! {
                span {
                    "User Area"
                }
            },
        ))
        .with_item(TopLevelNavigationBarItem::new(
            "https://discord.com/invite/W7Eqqj8NG2",
            html! {
                span {
                    "Discord Server"
                }
            },
        ));

    let footer = Footer::new(html! {
        "The Clicksync Challenge list and Pointercrate are in no way affiliated with RobTopGamesAB ® or eachother."
    })
    .with_column(FooterColumn::LinkList {
        heading: "The Clicksync Challenge list v1.5",
        links: vec![
            Link::new("/list/1/", "Hardest Challenge"),
            Link::new("/list/statsviewer/", "Stats Viewer"),
            Link::new("/account/", "User Area"),
        ],
    })
    .with_link("https://twitter.com/stadust1971", "Site Dev")
    .with_link("https://discord.com/invite/W7Eqqj8NG2", "Discord Server");

    PageConfiguration::new("Clicksync Challenge List", nav_bar, footer)
        .author("sphericle")
        .keywords("Your SEO keywords here")
}

#[shuttle_runtime::main]
async fn main(#[shuttle_runtime::Secrets] secrets: SecretStore) -> shuttle_rocket::ShuttleRocket {
    let rocket = configure_rocket(&secrets).await.expect("Failed to configure Rocket");

    rocket::build();
    Ok(rocket.into())
}
