use maud::{html, Markup, PreEscaped};
use pointercrate_core_pages::util::{dropdown, filtered_paginator, simple_dropdown};
use pointercrate_demonlist::nationality::Nationality;

pub mod individual;
pub mod national;

pub(crate) fn stats_viewer_panel() -> Markup {
    html! {
        section#stats.panel.fade.js-scroll-anim data-anim = "fade" {
            div.underlined {
                h2 {
                    "Stats Viewer:"
                }
            }
            p {
                "Get a detailed overview of who completed the most, created the most demons or beat the hardest demons! There is even a leaderboard to compare yourself to the very best!"
            }
            a.blue.hover.button#show-stats-viewer href = "/demonlist/statsviewer/ "{
                "Open the stats viewer!"
            }
        }
    }
}

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

fn hide_subdivision_panel() -> Markup {
    html! {
        section.panel.fade {
            h3.underlined {
                "Show subdivisions"
            }
            p {
                "Whether the map should display political subdivisions"
            }
            div.cb-container.flex.no-stretch style="margin-bottom:10px" {
                i {"Show political subdivisions"}
                input#show-subdivisions-checkbox type = "checkbox" checked="";
                span.checkmark {}
            }
        }
    }
}

struct StatsViewerRow(Vec<(&'static str, &'static str)>);

fn standard_stats_viewer_rows() -> Vec<StatsViewerRow> {
    vec![
        StatsViewerRow(vec![("Demonlist rank", "rank"), ("Demonlist score", "score")]),
        StatsViewerRow(vec![("Demonlist stats", "stats"), ("Hardest demon", "hardest")]),
        StatsViewerRow(vec![("Demons completed", "beaten")]),
        StatsViewerRow(vec![
            ("Demons created", "created"),
            ("Demons published", "published"),
            ("Demons verified", "verified"),
        ]),
        StatsViewerRow(vec![("Progress on", "progress")]),
    ]
}

fn stats_viewer_html(nations: Option<&[Nationality]>, rows: Vec<StatsViewerRow>) -> Markup {
    html! {
        section.panel.fade#statsviewer style="overflow:initial" {
            h2.underlined.pad {
                "Stats Viewer"
                @if let Some(nations) = nations {
                    " - "
                    (dropdown("International",
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
                            @for row in rows {
                                div.stats-container.flex.space {
                                    @for column in row.0 {
                                        span {
                                            b {
                                                (column.0)
                                            }
                                            br;
                                            span#(column.1) {}
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
