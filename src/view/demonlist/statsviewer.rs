use crate::{model::nationality::Nationality, view::filtered_paginator, view::simple_dropdown};
use maud::{html, Markup, PreEscaped};

mod heatmap;
pub(super) mod individual;
pub(super) mod nationbased;

fn continent_panel() -> Markup {
    html! {
        section.panel.fade style="overflow:initial"{
            h3.underlined {
                "Continent"
            }
            p {
                "Select a continent below to focus the stats viewer to that continent. Select 'All' to reset selection."
            }
            (simple_dropdown("continent-dropdown", Some("All"), vec!["Asia", "Europe", "Australia", "Africa", "North America", "South America", "Central America"].into_iter()))
        }
    }
}

fn stats_viewer_html(nations: Option<&[Nationality]>) -> Markup {
    html! {
        section.panel.fade#statsviewer style="overflow:initial" {
            h2.underlined.pad {
                "Stats Viewer"
                @if let Some(nations) = nations {
                    " - "
                    (super::super::dropdown("International",
                        html! {
                            li.white.hover.underlined data-value = "International" data-display = "International" {
                                span.em.em-world_map {}
                                (PreEscaped("&nbsp;"))
                                b {"WORLD"}
                                br;
                                span style = "font-size: 90%; font-style: italic" { "International" }
                            }
                        },
                        nations.iter().map(|nation| html! {
                            li.white.hover data-value = {(nation.iso_country_code)} data-display = {(nation.nation)} {
                                span class = "flag-icon" style={"background-image: url(/static2/images/flags/" (nation.iso_country_code.to_lowercase()) ".svg"} {}
                                (PreEscaped("&nbsp;"))
                                b {(nation.iso_country_code)}
                                br;
                                span style = "font-size: 90%; font-style: italic" {(nation.nation)}
                            }
                        })
                    ))
                }
            }
            div.flex.viewer {
                (filtered_paginator("stats-viewer-pagination", "/api/v1/players/ranking/"))
                p.viewer-welcome {
                    "Click on a player's name on the left to get started!"
                }
                div.viewer-content {
                    div {
                        div.flex.col {
                            h3#player-name style = "font-size:1.4em; overflow: hidden" {}
                            div.stats-container.flex.space {
                                span {
                                    b {
                                        "Demonlist rank:"
                                    }
                                    br;
                                    span#rank {}
                                }
                                span {
                                    b {
                                        "Demonlist score:"
                                    }
                                    br;
                                    span#score {}
                                }
                            }
                            div.stats-container.flex.space {
                                span {
                                    b {
                                        "List Stats:"
                                    }
                                    br;
                                    span#amount-beaten {}
                                }
                                span {
                                    b {
                                        "Hardest demon:"
                                    }
                                    br;
                                    span#hardest {}
                                }
                            }
                            div.stats-container.flex.space {
                                span {
                                    b {
                                        "Demons completed:"
                                    }
                                    br;
                                    span#beaten {}
                                }
                            }
                            div.stats-container.flex.space {
                                span {
                                    b {
                                        "List demons created:"
                                    }
                                    br;
                                    span#created {}
                                }
                                span {
                                    b {
                                        "List demons published:"
                                    }
                                    br;
                                    span#published {}
                                }
                                span {
                                    b {
                                        "List demons verified:"
                                    }
                                    br;
                                    span#verified {}
                                }
                            }
                            div.stats-container.flex.space {
                                span {
                                    b {
                                        "Progress on:"
                                    }
                                    br;
                                    span#progress {}
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
