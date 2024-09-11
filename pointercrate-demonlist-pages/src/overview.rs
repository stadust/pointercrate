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
use pointercrate_demonlist::{
    config as list_config,
    demon::{Demon, TimeShiftedDemon},
};

pub struct OverviewPage {
    pub team: Team,
    pub demonlist: Vec<Demon>,
    pub time_machine: Tardis,
    pub submitter_initially_visible: bool,
}

fn demon_panel(demon: &Demon, current_position: Option<i16>) -> Markup {
    html! {
         section.panel.fade style="overflow:hidden" {
             div.flex style = "align-items: center" {
                 div.thumb."ratio-16-9"."js-delay-css" style = "position: relative" data-property = "background-image" data-property-value = {"url('" (demon.thumbnail) "')"} {
                     @if let Some(video) = &demon.video {
                         a.play href = (video) {}
                     }
                     @else {
                         a.play href = "https://www.youtube.com/watch?v=dQw4w9WgXcQ" {}
                     }
                 }
                 div style = "padding-left: 15px" {
                     h2 style = "text-align: left; margin-bottom: 0px" {
                         a href = {"/list/permalink/" (demon.base.id) "/"} {
                             "#" (demon.base.position) (PreEscaped(" &#8211; ")) (demon.base.name)
                         }
                     }
                     h3 style = "text-align: left" {
                         i {
                             (demon.publisher.name)
                         }
                         @if let Some(current_position) = current_position {
                             br;
                             @if current_position > list_config::extended_list_size() {
                                 "Currently Legacy"
                             }
                             @else {
                                 "Currently #"(current_position)
                             }
                         }
                     }
                 }
             }
         }
    }
}

impl From<OverviewPage> for PageFragment {
    fn from(page: OverviewPage) -> Self {
        PageFragment::new("The CSCL", "The Clicksync Challenge List!")
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
                                    "@id": "https://cscl.shuttleapp.rs/",
                                    "name": "cscl"
                                }
                            },
                            {
                                "@type": "ListItem",
                                "position": 2,
                                "item": {
                                    "@id": "https://cscl.shuttleapp.rs/list/",
                                    "name": "list"
                                }
                            }
                        ]
                    },
                    "name": "The CSCL",
                    "description": "The Clicksync Challenge List!",
                    "url": "https://cscl.shuttleapp.rs/list/"
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
            link ref = "canonical" href = "https://cscl.shuttleapp.rs/list/";
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
                                    (demon_panel(current_demon, Some(*position_now)))
                                }
                            }
                        },
                        _ => {
                            @for demon in &self.demonlist {
                                @if demon.base.position <= list_config::extended_list_size() {
                                    (demon_panel(demon, None))
                                }
                            }
                        }
                    }
                }

                aside.right {
                    (self.team)
                    (super::rules_panel())
                    (super::nongs_panel())
                    (submit_panel())
                    (stats_viewer_panel())

                }
            }
        }
    }
}
