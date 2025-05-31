use crate::components::P;
use crate::{
    components::{
        submitter::{submit_panel, RecordSubmitter},
        team::Team,
        time_machine::Tardis,
    },
    statsviewer::stats_viewer_panel,
};
use maud::{html, Markup, PreEscaped};
use pointercrate_core_pages::{head::HeadLike, PageFragment};
use pointercrate_demonlist::player::FullPlayer;
use pointercrate_demonlist::{
    config as list_config, config,
    demon::{Demon, TimeShiftedDemon},
};

pub struct OverviewPage {
    pub team: Team,
    pub demonlist: Vec<Demon>,
    pub time_machine: Tardis,
    pub submitter_initially_visible: bool,
    pub claimed_player: Option<FullPlayer>,
}

impl From<OverviewPage> for PageFragment {
    fn from(page: OverviewPage) -> Self {
        PageFragment::new("Geometry Dash Demonlist", "The official pointercrate Demonlist!")
            .module("/static/core/js/modules/form.js")
            .module("/static/demonlist/js/modules/demonlist.js")
            .module("/static/demonlist/js/demonlist.js")
            .stylesheet("/static/demonlist/css/demonlist.css")
            .stylesheet("/static/core/css/sidebar.css")
            .head(page.head())
            .body(page.body())
    }
}

impl OverviewPage {
    fn head(&self) -> Markup {
        html! {
            (PreEscaped(r#"
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
            (PreEscaped(format!("
                <script>
                    window.list_length = {0};
                    window.extended_list_length = {1}
                </script>", list_config::list_size(), list_config::extended_list_size())
            ))
            // FIXME: abstract away
            link ref = "canonical" href = "https://pointercrate.com/demonlist/";
        }
    }

    fn body(&self) -> Markup {
        let demons_for_dropdown: Vec<&Demon> = match self.time_machine {
            Tardis::Activated { ref demons, .. } => demons.iter().map(|demon| &demon.current_demon).collect(),
            _ => self.demonlist.iter().collect(),
        };

        let dropdowns = super::dropdowns(&demons_for_dropdown[..], None);

        html! {
            (dropdowns)

            div.flex.m-center.container {
                main.left {
                    (self.time_machine)
                    (RecordSubmitter::new(self.submitter_initially_visible, &self.demonlist))

                    @match &self.time_machine {
                        Tardis::Activated { demons, ..} => {
                            @for TimeShiftedDemon {current_demon, position_now} in demons {
                                @if current_demon.base.position <= list_config::extended_list_size() {
                                    (self.demon_panel(current_demon, Some(*position_now)))
                                }
                            }
                        },
                        _ => {
                            @for demon in &self.demonlist {
                                @if demon.base.position <= list_config::extended_list_size() {
                                    (self.demon_panel(demon, None))
                                }
                            }
                        }
                    }
                }

                aside.right {
                    (self.team)
                    (super::rules_panel())
                    (submit_panel())
                    (stats_viewer_panel())
                    (super::discord_panel())
                }
            }
        }
    }

    fn demon_panel(&self, demon: &Demon, current_position: Option<i16>) -> Markup {
        let video_link = demon.video.as_deref().unwrap_or("https://www.youtube.com/watch?v=dQw4w9WgXcQ");

        let progress = self
            .claimed_player
            .as_ref()
            .and_then(|player| {
                if player.verified.iter().any(|d| d.id == demon.base.id) {
                    Some(100)
                } else {
                    player.records.iter().find(|r| r.demon.id == demon.base.id).map(|r| r.progress)
                }
            })
            .unwrap_or_default();

        let total_score = format!("{:.2}", demon.score(100));
        let progress_score = format!("{:.2}", demon.score(progress));
        let minimal_score = format!("{:.2}", demon.score(demon.requirement));

        html! {
             section.panel.fade.flex.mobile-col.completed[progress==100] style="overflow:hidden" {
                 a.thumb."ratio-16-9"."js-delay-css" href = (video_link) style = "position: relative" data-property = "background-image" data-property-value = {"url('" (demon.thumbnail) "')"} {}
                 div.flex.demon-info style = "align-items: center" {
                     div.demon-byline {
                         h2 style = "text-align: left; margin-bottom: 0px" {
                             a href = {"/demonlist/permalink/" (demon.base.id) "/"} {
                                 "#" (demon.base.position) (PreEscaped(" &#8211; ")) (demon.base.name)
                             }
                         }
                         h3 style = "text-align: left" {
                             "published by " (P(&demon.publisher, None))
                         }
                         div style="text-align: left; font-size: 0.8em" {
                            @if let Some(current_position) = current_position {
                                 @if current_position > list_config::extended_list_size() {
                                     "Currently Legacy"
                                 }
                                 @else {
                                     "Currently #"(current_position)
                                 }
                            }
                            @else {
                                @if demon.base.position > config::list_size() {
                                    (total_score) " points"
                                }
                                @ else {
                                    (minimal_score) " (" (demon.requirement) "%) — " (total_score) " (100%) points"
                                }
                            }
                         }
                     }
                     @if progress > 0 && current_position.is_none() {
                        div.flex.col style = "font-weight: bold; text-align: right" {
                            span style = "font-size: 3em" { (progress) "%" }
                            span style = "font-size: 0.8em" { (progress_score) " points" }
                        }
                    }
                }
            }
        }
    }
}
