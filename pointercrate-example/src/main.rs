use maud::html;
use pointercrate_core::localization::LocalesLoader;
use pointercrate_core::pool::PointercratePool;
use pointercrate_core::{error::CoreError, localization::tr};
use pointercrate_core_api::{error::ErrorResponder, maintenance::MaintenanceFairing, preferences::PreferenceManager};
use pointercrate_core_macros::localized_catcher;
use pointercrate_core_pages::{
    footer::{Footer, FooterColumn, Link},
    navigation::{NavigationBar, TopLevelNavigationBarItem},
    PageConfiguration,
};
use pointercrate_demonlist::LIST_ADMINISTRATOR;
use pointercrate_demonlist_api::GeolocationProvider;
use pointercrate_demonlist_pages::account::{
    demons::DemonsTab, list_integration::ListIntegrationTab, players::PlayersPage, records::RecordsPage,
};
use pointercrate_user::MODERATOR;
use pointercrate_user_pages::account::{profile::ProfileTab, users::UsersTab, AccountPageConfig};
use rocket::{async_trait, fs::FileServer, response::Redirect, serde, uri, Request};
use std::net::IpAddr;
use unic_langid::lang;
use unic_langid::subtags::Language;

/// A catcher for 404 errors (e.g. when a user tried to navigate to a URL that
/// does not exist)
///
/// An [`ErrorResponder`] will return either a JSON or an HTML error page,
/// depending on what `Accept` headers are set on the request.
#[localized_catcher]
#[rocket::catch(404)]
async fn catch_404() -> ErrorResponder {
    // `CoreError` contains various generic error conditions that might happen
    // anywhere across the website. `CoreError::NotFound` is a generic 404 NOT FOUND
    // error with code 40400.
    CoreError::NotFound.into()
}

/// Failures in json deserialization of request bodies will just return
/// an immediate 422 response. This catcher is needed to translate them into a pointercrate
/// error response.
#[localized_catcher]
#[rocket::catch(422)]
async fn catch_422() -> ErrorResponder {
    CoreError::UnprocessableEntity.into()
}

/// Failures from the authorization FromRequest implementations can return 401s
#[localized_catcher]
#[rocket::catch(401)]
async fn catch_401() -> ErrorResponder {
    CoreError::Unauthorized.into()
}

/// We do not have a home page, so have the website root simply redirect to the demonlist
#[rocket::get("/")]
fn home() -> Redirect {
    Redirect::to(uri!("/demonlist/"))
}

const DEFAULT_LOCALE: Language = lang!("en");

/// A very simplistic geolocation provider based on https://ipwho.is/
///
/// Note that ipwho.is is only free for testing, non-commercial use-cases, and
/// up to 1000 requests / mo. In a production environment, it would be up to you to
/// implement appropriate rate limits / use a service that matches your usecase!
///
/// Note that when running this locally, all requests will come from 127.0.0.1, which
/// obviously cannot be geolocated.
struct IpWhoIsGeolocationProvider;

#[async_trait]
impl GeolocationProvider for IpWhoIsGeolocationProvider {
    async fn geolocate(&self, req: &Request<'_>) -> Option<(String, Option<String>)> {
        #[derive(serde::Deserialize)]
        struct IpWhoIsResponse {
            country_code: String,
            region_code: Option<String>,
        }

        let remote_ip: IpAddr = req.guard().await.succeeded()?;

        let resp = reqwest::get(format!("https://ipwho.is/{}", remote_ip)).await.ok()?;

        let data: IpWhoIsResponse = resp.json().await.ok()?;

        Some((data.country_code, data.region_code))
    }
}

#[rocket::launch]
async fn rocket() -> _ {
    // Load the configuration from your .env file
    dotenv::dotenv().unwrap();

    // Load the translation files
    LocalesLoader::load(&[
        "pointercrate-core-pages/static/ftl/",
        "pointercrate-demonlist-pages/static/ftl/",
        "pointercrate-user-pages/static/ftl/",
        "pointercrate-example/static/ftl/",
    ])
    .expect("Failed to load localization files")
    .commit(DEFAULT_LOCALE);

    // Initialize a database connection pool to the database specified by the
    // DATABASE_URL environment variable
    let pool = PointercratePool::init().await;

    // Set up the HTTP server
    let rocket = rocket::build()
        // Tell it about the connection pool to use (individual handlers can get hold of this pool by declaring an argument of type `&State<PointercratePool>`)
        .manage(pool)
        // Tell pointercrate's core components about navigation bar and footers, so that it knows how to render the website
        // We are passing is as a function pointer so the page can load it in a different language each time a page is rendered
        .manage(page_configuration as fn() -> PageConfiguration)
        // Register our 404 catcher
        .register("/", rocket::catchers![catch_401, catch_404, catch_422])
        // Register our home page
        .mount("/", rocket::routes![home]);

    // Define the permissions in use on our website. We just use the default setup
    // from `pointercrate_user` and `pointercrate_demonlist`, but if you for example
    // do not want list administrators to be able to promote helpers to moderators
    // in autonomy, you could use a custom [`PermissionsManager`] where the
    // `LIST_ADMINISTRATOR` permission does not assign `LIST_MODERATOR` and
    // `LIST_HELPER`. For more information on pointercrate' permissions system, see
    // the documentation of the [`PermissionsManager`] structure.
    let mut permissions_manager = pointercrate_user::default_permissions_manager();
    permissions_manager.merge_with(pointercrate_demonlist::default_permissions_manager());

    let rocket = rocket.manage(permissions_manager);

    // Define the preferences our website supports. Preferences are sent to us from
    // the client via cookies.
    let preference_manager = PreferenceManager::default().with_localization();

    let rocket = rocket.manage(preference_manager);

    // Register the geolocation provider, so that we can geolocate player claims. The type erasure is important, otherwise you'll get internal server errors!
    let rocket = rocket.manage(Box::new(IpWhoIsGeolocationProvider) as Box<dyn GeolocationProvider>);

    // Set up which tabs can show up in the "user area" of your website. Anything
    // that implements the [`AccountPageTab`] trait can be displayed here. Note that
    // tabs will only be visible for users for which
    // [`AccountPageTab::should_display_for`] returns `true`.
    let account_page_config = AccountPageConfig::default()
        // Tab where users can modify their own accounts
        .with_page(ProfileTab)
        // Tab where users can initiate player claims and manage their claimed players
        .with_page(ListIntegrationTab("https://discord.gg/tMBzYP77ag"))
        // Tab where website moderators can manage permissions. 
        // The vector below specified which permissions a user needs to have for the tab to be displayed.
        .with_page(UsersTab(vec![MODERATOR, LIST_ADMINISTRATOR]))
        // Tab where list helpers can manage demons
        .with_page(DemonsTab)
        // Tab where list helpers can manage players
        .with_page(PlayersPage)
        // Tab where list helpers can manage records
        .with_page(RecordsPage);

    let rocket = rocket.manage(account_page_config);

    // Changing `false` to `true` here will put your website into "maintenance mode", which will disable all mutating request handlers and always return 503 SERVICE UNAVAILABLE responses for non-GET requests.
    let rocket = rocket.attach(MaintenanceFairing::new(false));

    // Register all the endpoints related to the demonlist to our server (this is
    // optional, but without registering the demonlist related endpoint your website
    // will just be User Account Simulator 2024).
    let rocket = pointercrate_demonlist_api::setup(rocket);

    // Register all the endpoints related to the user account system to our server
    let rocket = pointercrate_user_api::setup(rocket);

    // Let rocket serve static files (e.g. CSS, JavaScript, images, etc.). In a
    // production environment, you will not want rocket to be responsible for this
    // and instead use a web server such as nginx as a reverse proxy to serve your
    // static files.

    rocket
        .mount("/static/core", FileServer::new("pointercrate-core-pages/static"))
        .mount("/static/demonlist", FileServer::new("pointercrate-demonlist-pages/static"))
        .mount("/static/user", FileServer::new("pointercrate-user-pages/static"))
        .mount("/static/example", FileServer::new("pointercrate-example/static"))
}

/// Constructs a [`PageConfiguration`] for your site.
///
/// A `PageConfiguration` object is a description of your websites general
/// look-and-feel. It defines the navigation bar and footer layouts (e.g. what
/// links to include) and various metadata without you needing to worry (much)
/// about styling and layout.
fn page_configuration() -> PageConfiguration {
    // Define a navigation bar with only two items, a link to the user account page,
    // and a link to your demonlist.
    let nav_bar = NavigationBar::new("/static/images/path/to/your/logo.png")
        .with_item(
            TopLevelNavigationBarItem::new(
                Some("/demonlist/"),
                // Pointercrate uses the "maud" create as its templating engine. 
                // It allows you to describe HTML via Rust macros that allow you to dynamically generate content using
                // a Rust-like syntax and by interpolating and Rust variables from surrounding scopes (as long as the
                // implement the `Render` trait). See https://maud.lambda.xyz/ for details.
                html! {
                    span {
                        (tr("nav-demonlist"))
                    }
                },
            )
            // Add a drop down to the demonlist item, just like on pointercrate.com
            .with_sub_item(Some("/demonlist/statsviewer/"), html! { (tr("nav-demonlist.stats-viewer")) })
            .with_sub_item(Some("/demonlist/?submitter=true"), html! { (tr("nav-demonlist.record-submitter")) })
            .with_sub_item(Some("/demonlist/?timemachine=true"), html! { (tr("nav-demonlist.time-machine")) }),
        )
        .with_item(TopLevelNavigationBarItem::new(Some("/login/"), {
            html! {
                span {
                    (tr("nav-userarea"))
                }
            }
        }));

    // A footer consists of a copyright notice, an arbitrary amount of columns
    // displayed below it, side-by-side, and potentially some social media links to
    // your team
    let footer = Footer::new(html! {
        "© Copyright <year> <your website>"
        br;
        "All rights reserved"
        br;
        "<your website> and <your demonlist> are in no way affiliated with RobTopGamesAB ® or pointercrate.com"
    })
    // Add a column with links for various list-related highlights
    .with_column(FooterColumn::LinkList {
        heading: tr("footer-demonlist"),
        links: vec![
            Link::new("/demonlist/1/", tr("footer-demonlist.top-demon")),
            Link::new(
                format!("/demonlist/{}/", pointercrate_demonlist::config::list_size() + 1),
                tr("footer-demonlist.extended-list"),
            ),
            Link::new(
                format!("/demonlist/{}/", pointercrate_demonlist::config::extended_list_size() + 1),
                tr("footer-demonlist.legacy-list"),
            ),
        ],
    })
    // Some links to social media, for example your twitter
    .with_link("https://twitter.com/stadust1971", tr("footer-tweet.developer"));

    // Stitching it all together into a page configuration
    PageConfiguration::new("<your website name here>", nav_bar, footer)
        // Used for the HTML "author" meta tag
        .author("your name")
        // Used for the HTML "keywords" meta tag
        .keywords("Your SEO keywords here")
}
