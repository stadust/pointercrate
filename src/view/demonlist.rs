use super::{url_helper, Page};
use crate::{
    actor::{database::GetDemonlistOverview, http::GetDemon},
    config::{EXTENDED_LIST_SIZE, LIST_SIZE},
    error::PointercrateError,
    model::{
        demon::{Demon, PartialDemon},
        user::User,
    },
    state::PointercrateState,
    video,
};
use actix_web::{AsyncResponder, FromRequest, HttpRequest, Path, Responder};
use gdcf::model::{Creator, PartialLevel};
use maud::{html, Markup, PreEscaped};
use tokio::prelude::{Future, IntoFuture};

struct ListSection {
    name: &'static str,
    description: &'static str,
    id: &'static str,
    numbered: bool,
}

static MAIN_SECTION: ListSection = ListSection {
    name: "Main List",
    description: "The main section of the demonlist. These demons are the hardest rated levels in the game. Records are accepted above a given threshold and award a large amount of points!",
    id: "mainlist",
    numbered: true,
};

static EXTENDED_SECTION: ListSection = ListSection {
    name: "Extended List",
    description: "These are demons that dont qualify for the main section of the list, but are still of high relevance. Only 100% records are accepted for these demons! Note that non-100% that were submitted/approved before a demon fell off the main list will be retained",
    id: "extended",
    numbered: true
};

static LEGACY_SECTION: ListSection  = ListSection{
    name: "Legacy List",
    description: "These are demons that used to be in the top 100, but got pushed off as new demons were added. They are here for nostalgic reasons. This list is in no order whatsoever and will not be maintained any longer at all. This means no new records will be added for these demons.",
    id: "legacy",
    numbered: false,
};

#[derive(Debug)]
pub struct DemonlistOverview {
    pub demon_overview: Vec<PartialDemon>,
    pub admins: Vec<User>,
    pub mods: Vec<User>,
    pub helpers: Vec<User>,
}

pub fn overview_handler(req: &HttpRequest<PointercrateState>) -> impl Responder {
    let req_clone = req.clone();

    req.state()
        .database(GetDemonlistOverview)
        .map(move |overview| overview.render(&req_clone))
        .responder()
}

impl Page for DemonlistOverview {
    fn title(&self) -> String {
        "Geometry Dash Demonlist".to_string()
    }

    fn description(&self) -> String {
        "The official Geometry Dash Demonlist on pointercrate!".to_string()
    }

    fn scripts(&self) -> Vec<&str> {
        vec![]
    }

    fn stylesheets(&self) -> Vec<&str> {
        vec!["css/demonlist.v2.1.css"]
    }

    fn body(&self, req: &HttpRequest<PointercrateState>) -> Markup {
        let dropdowns = dropdowns(req, &self.demon_overview, None);

        html! {
            (dropdowns)

            div.flex.m-center#container {
                div.left {
                    div.panel.fade {
                        div.underlined {
                            h1 {"Demonlist"}
                        }
                        p {
                            "This page provides an view of the entire demonlist. To get further information, like the list of records or the level password, click on a demons name to get to its page! Note that this page can be slow to load due to the abundance of videos"
                        }
                    }
                    (rules_panel())
                    @for demon in &self.demon_overview {
                        div.panel.fade.js-scroll-anim data-anim="fade" {
                            div.underlined {
                                h2 {
                                    a href = {"/demonlist/" (demon.position)} {
                                        "#" (demon.position) " - " (demon.name) " by " (demon.publisher)
                                    }
                                }
                            }
                            @if let Some(ref video) = demon.video {
                                iframe."ratio-16-9" style="width: 90%; margin: 15px 5% 0px" src = (video::embed(video)) {}
                            }
                        }
                    }
                }
                (sidebar(&self.admins, &self.mods, &self.helpers))
            }

        }
    }

    fn head(&self, _: &HttpRequest<PointercrateState>) -> Vec<Markup> {
        vec![html! {
            (PreEscaped(r#"
                <script type="application/ld+json">
                {{
                    "@context": "http://schema.org",
                    "@type": "WebPage",
                    "breadcrumb": {{
                        "@type": "BreadcrumbList",
                        "itemListElement": [{{
                                "@type": "ListItem",
                                "position": 1,
                                "item": {{
                                    "@id": "https://pointercrate.com/",
                                    "name": "pointercrate"
                                }}
                            }},{{
                                "@type": "ListItem",
                                "position": 2,
                                "item": {{
                                    "@id": "https://pointercrate.com/demonlist/",
                                    "name": "demonlist"
                                }}
                            }}
                        ]
                    }},
                    "name": "Geometry Dash Demonlist",
                    "description": "The official Geometry Dash Demonlist on pointercrate!",
                    "url": "https://pointercrate.com/demonlist/"
                }}
                </script>
            "#))
        }]
    }
}

#[derive(Debug)]
pub struct Demonlist {
    overview: DemonlistOverview,
    current_demon: Demon,
    server_level: Option<PartialLevel<u64, Creator>>,
}

pub fn handler(req: &HttpRequest<PointercrateState>) -> impl Responder {
    let req_clone = req.clone();
    let state = req.state().clone();

    Path::<i16>::extract(req)
        .map_err(|_| PointercrateError::bad_request("Demon position must be integer"))
        .into_future()
        .and_then(move |position| {
            state
                .get(position.into_inner())
                .and_then(move |current_demon: Demon| {
                    state
                        .database(GetDemonlistOverview)
                        .and_then(move |overview| {
                            state
                                .gdcf
                                .send(GetDemon(current_demon.name.clone()))
                                .map_err(PointercrateError::internal)
                                .map(move |demon| {
                                    Demonlist {
                                        overview,
                                        current_demon,
                                        server_level: demon,
                                    }
                                    .render(&req_clone)
                                })
                        })
                })
        })
        .responder()
}

impl Page for Demonlist {
    fn title(&self) -> String {
        format!(
            "#{} - {} - Geometry Dash Demonlist",
            self.current_demon.position, self.current_demon.name
        )
    }

    fn description(&self) -> String {
        if let Some(ref level) = self.server_level {
            if let Some(ref description) = level.description {
                return format!("{}: {}", self.title(), description)
            }
        }
        format!("{}: <No Description Provided>", self.title())
    }

    fn scripts(&self) -> Vec<&str> {
        vec![]
    }

    fn stylesheets(&self) -> Vec<&str> {
        vec!["css/demonlist.v2.1.css"]
    }

    fn body(&self, req: &HttpRequest<PointercrateState>) -> Markup {
        let dropdowns = dropdowns(
            req,
            &self.overview.demon_overview,
            Some(&self.current_demon),
        );

        html! {
            (dropdowns)
        }
    }

    fn head(&self, _: &HttpRequest<PointercrateState>) -> Vec<Markup> {
        vec![html! {
            (PreEscaped(format!(r#"
                <script type="application/ld+json">
                {{
                    "@context": "http://schema.org",
                    "@type": "WebPage",
                    "breadcrumb": {{
                        "@type": "BreadcrumbList",
                        "itemListElement": [{{
                                "@type": "ListItem",
                                "position": 1,
                                "item": {{
                                    "@id": "https://pointercrate.com/",
                                    "name": "pointercrate"
                                }}
                            }},{{
                                "@type": "ListItem",
                                "position": 2,
                                "item": {{
                                    "@id": "https://pointercrate.com/demonlist/",
                                    "name": "demonlist"
                                }}
                            }},{{
                                "@type": "ListItem",
                                "position": 3,
                                "item": {{
                                    "@id": "https://pointercrate.com/demonlist/{0}/",
                                    "name": "{1}"
                                }}
                            }}
                        ]
                    }},
                    "name": "\#{0} - {1}",
                    "description": {2},
                    "url": "https://pointercrate.com/demonlist/{0}/"
                }}
                </script>
            "#, self.current_demon.position, self.current_demon.name, self.description())))
        }]
    }
}

fn dropdowns(
    req: &HttpRequest<PointercrateState>, all_demons: &[PartialDemon], current: Option<&Demon>,
) -> Markup {
    let (main, extended, legacy) = if all_demons.len() < *LIST_SIZE as usize {
        (&all_demons[..], Default::default(), Default::default())
    } else {
        let (extended, legacy) = if all_demons.len() < *EXTENDED_LIST_SIZE as usize {
            (&all_demons[*LIST_SIZE as usize..], Default::default())
        } else {
            (
                &all_demons[*LIST_SIZE as usize..*EXTENDED_LIST_SIZE as usize],
                &all_demons[*EXTENDED_LIST_SIZE as usize..],
            )
        };

        (&all_demons[..*LIST_SIZE as usize], extended, legacy)
    };

    html! {
        div.flex.wrap.m-center.fade#lists style="text-align: center;" {
            // The drop down for the main list:
            (dropdown(req, &MAIN_SECTION, main, current))
            // The drop down for the extended list:
            (dropdown(req, &EXTENDED_SECTION, extended, current))
            // The drop down for the legacy list:
            (dropdown(req, &LEGACY_SECTION, legacy, current))
        }
    }
}

fn dropdown(
    req: &HttpRequest<PointercrateState>, section: &ListSection, demons: &[PartialDemon],
    current: Option<&Demon>,
) -> Markup {
    let format = |demon: &PartialDemon| -> Markup {
        html! {
            @if section.numbered {
                a href = (url_helper::demon(req, demon.position)) {
                    {"#" (demon.position) " - " (demon.name)}
                    br ;
                    (demon.publisher)
                }
            }
            @else {
                a href = (url_helper::demon(req, demon.position)) {
                    {(demon.name)}
                    br ;
                    (demon.publisher)
                }
            }
        }
    };

    html! {
        div {
            div.button.white.hover.no-shadow.js-toggle data-toggle-group="0" onclick={"javascript:void(DropDown.toggleDropDown('" (section.id) "'))"} {
                (section.name)
            }

            div.see-through.fade.dropdown#(section.id) {
                div.search.seperated {
                    input placeholder = "Filter..." {}
                }
                p style = "margin: 10px" {
                    (section.description)
                }
                ul.flex.wrap.space {
                    @for demon in demons {
                        @match current {
                            Some(current) if current.position == demon.position =>
                                li.hover.white.active title={"#" (demon.position) " - " (demon.name)} {
                                    (format(demon))
                                },
                            _ =>
                                li.hover.white title={"#" (demon.position) " - " (demon.name)} {
                                    (format(demon))
                                }
                        }
                    }
                }
            }
        }
    }
}

fn sidebar(admins: &[User], mods: &[User], helpers: &[User]) -> Markup {
    html! {
        div.right {
            (team_panel(admins, mods, helpers))
            (submit_panel())
            (stats_viewer_panel())
            (discord_panel())
        }
    }
}

fn team_panel(admins: &[User], mods: &[User], helpers: &[User]) -> Markup {
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
        div.panel.fade.js-scroll-anim#editors data-anim = "fade" {
            div.underlined {
                h2 {
                    "List Editors:"
                }
            }
            p {
                "Contact any of these people if you have problems with the list or want to see a specific thing changed."
            }
            ul style = "line-height: 30px" {
                @for admin in admins {
                    b {
                        (maybe_link(admin))
                    }
                }
                @for moderator in mods {
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
                @for helper in helpers {
                    (maybe_link(helper))
                }
            }
        }
    }
}

fn rules_panel() -> Markup {
    html! {
        did#rules.panel.fade.flex.js-scroll-anim data-anim = "fade" style = "flex-direction: column" {
            div.underlined {
                h2 {
                    "Rules:"
                }
            }
            ul.roman {
                li {
                    span {
                        "Anyone posting illegitimate recordings (hacked, cut, stolen, automated gameplay, no-clip, etc.) and passing them of as legit will have all their records removed from this list"
                    }
                }
                li {
                    span {
                        "Demons need to be rated to be included on this list"
                    }
                }
                li {
                    span {
                        "If you verified a level on this list, your record for it won't be included - You get points for your verification though"
                    }
                }
                li {
                    span {
                        "If a record has been added, it is legit and was either streamed or has a full video uploaded"
                    }
                }
                li {
                    span {
                        "The record holder must meet the percentage requirement in order to be added to the list for that level"
                    }
                }
                li {
                    span {
                        "Be polite about suggesting changes. We probably won't listed to you if you're rude or forceful about it"
                    }
                }
                li {
                    span {
                        "Being in a group in which people beat levels for the same channel, yet passing that channel of as being a single person's, can cause your records to be temporarily removed from this list"
                    }
                }
                li {
                    span {
                        "Records made using the FPS bypass are"
                        i { " not " }
                        "accepted"
                    }
                }
            }
        }
    }
}

fn submit_panel() -> Markup {
    html! {
        div#submit.panel.fade.js-scroll-anim data-anim = "fade" {
            div.underlined {
                h2 {
                    "Submit Records:"
                }
            }
            p {
                "Note: Please do not submit nonsense, it only makes it harder for us all and will get you banned. Also note that the form rejects duplicate submission"
            }
            a.blue.hover.button.slightly-rounded.js-scroll data-destination = "submitter" data-reveal = "true" {
                "Submit a record!"
            }
        }
    }
}

fn stats_viewer_panel() -> Markup {
    html! {
        div#stats.panel.fade.js-scroll-anim data-anim = "fade" {
            div.underlined {
                h2 {
                    "Stats Viewer"
                }
            }
            p {
                "Get a detailed overview of who completed the most, created the most demons or beat the hardest demons! There is even a leaderboard to compare yourself to the very best!"
            }
            a.blue.hover.button.slightly-rounded.js-scroll data-destination = "statsviewer" data-reveal = "true" {
                "Open the stats viewer!"
            }
        }
    }
}

fn discord_panel() -> Markup {
    html! {
        div.panel.fade.js-scroll-anim data-anim = "fade" {
            iframe#discord style = "width: 100%; height: 400px;" allowtransparency="true" frameborder = "0" {}
            p {
                "Join the official demonlist discord server, where you can get in touch with the demonlist team!"
            }
        }
    }
}
