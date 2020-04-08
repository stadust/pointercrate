use crate::{
    config,
    model::{nationality::Nationality, user::User},
    permissions::Permissions,
    state::PointercrateState,
    video,
    view::Page,
    Result, ViewResult,
};
use actix_web::HttpResponse;
use actix_web_codegen::get;
use maud::{html, Markup, PreEscaped};
use sqlx::PgConnection;

#[derive(Debug)]
pub struct OverviewDemon {
    pub id: i32,
    pub position: i16,
    pub name: String,
    pub publisher: String,
    pub video: Option<String>,
}

#[derive(Debug)]
pub struct DemonlistOverview {
    pub demon_overview: Vec<OverviewDemon>,
    pub admins: Vec<User>,
    pub mods: Vec<User>,
    pub helpers: Vec<User>,
    pub nations: Vec<Nationality>,
}

pub async fn overview_demons(connection: &mut PgConnection) -> Result<Vec<OverviewDemon>> {
    Ok(sqlx::query_as!(
        OverviewDemon,
        "SELECT demons.id, position, demons.name::TEXT, CASE WHEN verifiers.link_banned THEN NULL ELSE video::TEXT END, \
         players.name::TEXT as publisher FROM demons INNER JOIN players ON demons.publisher = players.id INNER JOIN players AS verifiers \
         ON demons.verifier = verifiers.id WHERE position IS NOT NULL ORDER BY position"
    )
    .fetch_all(connection)
    .await?)
}

impl DemonlistOverview {
    pub(super) fn team_panel(&self) -> Markup {
        let maybe_link = |user: &User| -> Markup {
            html! {
                li {
                    @match user.youtube_channel {
                        Some(ref channel) => a target = "_blank" href = (channel) {
                            (user.name())
                        },
                        None => (user.name())
                    }
                }
            }
        };

        html! {
            section.panel.fade.js-scroll-anim#editors data-anim = "fade" {
                div.underlined {
                    h2 {
                        "List Editors:"
                    }
                }
                p {
                    "Contact any of these people if you have problems with the list or want to see a specific thing changed."
                }
                ul style = "line-height: 30px" {
                    @for admin in &self.admins {
                        b {
                            (maybe_link(admin))
                        }
                    }
                    @for moderator in &self.mods {
                        (maybe_link(moderator))
                    }
                }
                div.underlined {
                    h2 {
                        "List Helpers"
                    }
                }
                p {
                    "Contact these people if you have any questions regarding why a specific record was rejected. Do not needlessly bug them about checking submissions though!"
                }
                ul style = "line-height: 30px" {
                    @for helper in &self.helpers {
                        (maybe_link(helper))
                    }
                }
            }
        }
    }

    pub(super) async fn load(connection: &mut PgConnection) -> Result<DemonlistOverview> {
        let admins = User::by_permission(Permissions::ListAdministrator, connection).await?;
        let mods = User::by_permission(Permissions::ListModerator, connection).await?;
        let helpers = User::by_permission(Permissions::ListHelper, connection).await?;

        let nations = Nationality::all(connection).await?;
        let demon_overview = overview_demons(connection).await?;

        Ok(DemonlistOverview {
            admins,
            mods,
            helpers,
            nations,
            demon_overview,
        })
    }
}

#[get("/demonlist/")]
pub async fn index(state: PointercrateState) -> ViewResult<HttpResponse> {
    let mut connection = state.connection().await?;

    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(DemonlistOverview::load(&mut connection).await?.render().0))
}

impl Page for DemonlistOverview {
    fn title(&self) -> String {
        "Geometry Dash Demonlist".to_string()
    }

    fn description(&self) -> String {
        "The official pointercrate Demonlist!".to_string()
    }

    fn scripts(&self) -> Vec<&str> {
        vec!["js/modules/form.mjs", "js/modules/demonlist.mjs", "js/demonlist.v2.2.js"]
    }

    fn stylesheets(&self) -> Vec<&str> {
        vec!["css/demonlist.v2.1.css", "css/sidebar.css"]
    }

    fn body(&self) -> Markup {
        let dropdowns = super::dropdowns(&self.demon_overview, None);

        html! {
            (dropdowns)

            div.flex.m-center.container {
                main.left {
                    (super::submission_panel())
                    (super::stats_viewer(&self.nations))
                    @for demon in &self.demon_overview {
                        @if demon.position <= config::extended_list_size() {
                            section.panel.fade style="overflow:hidden" {
                                div.underlined.flex style = "padding-bottom: 10px; align-items: center" {
                                    @if let Some(ref video) = demon.video {
                                        div.thumb."ratio-16-9"."js-delay-css" style = "position: relative" data-property = "background-image" data-property-value = {"url('" (video::thumbnail(video)) "')"} {
                                            a.play href = (video) {}
                                        }
                                        div.leftlined.pad {
                                            h2 style = "text-align: left; margin-bottom: 0px" {
                                                a href = {"/demonlist/" (demon.position)} {
                                                    "#" (demon.position) " - " (demon.name)
                                                }
                                            }
                                            h3 style = "text-align: left" {
                                                i {
                                                    "by " (demon.publisher)
                                                }
                                            }
                                        }
                                    }
                                    @else {
                                        h2 {
                                            a href = {"/demonlist/" (demon.position)} {
                                                "#" (demon.position) " - " (demon.name) " by " (demon.publisher)
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                aside.right {
                    (self.team_panel())
                    (super::rules_panel())
                    (super::submit_panel())
                    (super::stats_viewer_panel())
                    (super::discord_panel())
                }
            }

        }
    }

    fn head(&self) -> Vec<Markup> {
        vec![
            html! {
            (PreEscaped(r#"
                <link href="https://cdnjs.cloudflare.com/ajax/libs/flag-icon-css/3.4.3/css/flag-icon.min.css" rel="stylesheet">
                <script type="application/ld+json">
                {
                    "@context": "http://schema.org",
                    "@type": "WebPage",
                    "breadcrumb": {
                        "@type": "BreadcrumbList",
                        "itemListElement": [
                            {
                                "@type": "ListItem",
                                "position": 1,
                                "item": {
                                    "@id": "https://pointercrate.com/",
                                    "name": "pointercrate"
                                }
                            },
                            {
                                "@type": "ListItem",
                                "position": 2,
                                "item": {
                                    "@id": "https://pointercrate.com/demonlist/",
                                    "name": "demonlist"
                                }
                            }
                        ]
                    },
                    "name": "Geometry Dash Demonlist",
                    "description": "The official pointercrate Demonlist!",
                    "url": "https://pointercrate.com/demonlist/"
                }
                </script>
            "#))
            },
            html! {
                (PreEscaped(format!("
                    <script>
                        window.list_length = {0};
                        window.extended_list_length = {1}
                    </script>", config::list_size(), config::extended_list_size())
                ))
            },
        ]
    }
}
