use super::{url_helper, Page};
use crate::{
    model::demon::{Demon, PartialDemon},
    state::PointercrateState,
};
use actix_web::HttpRequest;
use gdcf::model::{Creator, PartialLevel};
use maud::{html, Markup, PreEscaped};

#[derive(Debug)]
pub struct Demonlist {
    current_demon: Demon,
    all_demons: Vec<PartialDemon>,
    server_level: Option<PartialLevel<u64, Creator>>,
}

impl Demonlist {
    pub fn new(demon: Demon) -> Demonlist {
        Demonlist {
            current_demon: demon,
            all_demons: Vec::new(),
            server_level: None,
        }
    }
}

impl Page for Demonlist {
    fn title(&self) -> String {
        format!(
            "#{} - {} - Geometry Dash Demonlist",
            self.current_demon.position, self.current_demon.name
        )
    }

    fn description(&self) -> String {
        String::new()
        //self.current_demon.description.as_ref().unwrap_or("")
    }

    fn scripts(&self) -> Vec<&str> {
        vec![]
    }

    fn stylesheets(&self) -> Vec<&str> {
        vec![]
    }

    fn body(&self, req: &HttpRequest<PointercrateState>) -> Markup {
        html! {
            // The bar of dropdown lists at the top of the page:
            div.flex.wrap.m-center.fade#lists style="text-align: center;" {
                // The drop down for the main list:
                div.button.white.hover.no-shadow.js-toggle data-toggle-group="0" onclick="javascript:void(DropDown.toggleDropDown('mainlist'))" {
                    "Main List"
                }

                // Drop down content:
                div.see-through.fade.dropdown#mainlist {
                    div.search.seperated {
                        input placeholder="Filter" {}
                    }
                    ul.flex.wrap.space {
                        @for demon in &self.all_demons {
                            @if demon.position == self.current_demon.position {
                                li.hover.white.active title={"#" (demon.position) " - " (demon.name)} {
                                    a href = (url_helper::demon(req, demon.position)) {
                                        {"#" (demon.position) " - " (demon.name)}
                                    }
                                }
                            } else {
                                li.hover.white title={"#" (demon.position) " - " (demon.name)} {
                                    a href = (url_helper::demon(req, demon.position)) {
                                        {"#" (demon.position) " - " (demon.name)}
                                    }
                                }
                            }
                        }
                    }
                }
            }
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
    "description": "", // TODO: description
    "url": "https://pointercrate.com/demonlist/{0}/"
}}
</script>
            "#, self.current_demon.position, self.current_demon.name)))
        }]
    }
}

fn rules_panel() -> Markup {
    html! {
        did#rules.panel.fade.js-scroll-anim data-anim = "fade" {
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
                        "Records made using the FPS bypass are "
                        i { "not" }
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
            iframe#disccord style = "width: 100%; height: 400px;" allowtransparency="true" frameborder = "0" {}
            p {
                "Join the official demonlist discord server, where you can get in touch with the demonlist team!"
            }
        }
    }
}
