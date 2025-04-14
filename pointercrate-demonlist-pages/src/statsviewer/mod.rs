use maud::{html, Markup, PreEscaped};
use pointercrate_core::localization::tr;
use pointercrate_core_pages::util::{dropdown, filtered_paginator, simple_dropdown};
use pointercrate_demonlist::nationality::Nationality;

pub mod individual;
pub mod national;

pub(crate) fn stats_viewer_panel() -> Markup {
    html! {
        section #stats.panel.fade.js-scroll-anim data-anim = "fade" {
            div.underlined {
                h2 {
                    (tr("statsviewer-panel"))
                }
            }
            p {
                (tr("statsviewer-panel.info"))
            }
            a.blue.hover.button #show-stats-viewer href = "/demonlist/statsviewer/ "{
                (tr("statsviewer-panel.button"))
            }
        }
    }
}

fn continent_panel() -> Markup {
    html! {
        section.panel.fade style="overflow:initial"{
            h3.underlined {
                (tr("continent-panel"))
            }
            p {
                (tr("continent-panel.info"))
            }
            (simple_dropdown("continent-dropdown", Some("All"), vec!["Asia", "Europe", "Australia", "Africa", "North America", "South America", "Central America"].into_iter()))
        }
    }
}

fn demon_sorting_panel() -> Markup {
    html! {
        section.panel.fade style="overflow:initial" {
            h3.underlined {
                (tr("demon-sorting-panel"))
            }
            p {
                (tr("demon-sorting-panel.info"))
            }
            (simple_dropdown("demon-sorting-mode-dropdown", Some("Alphabetical"), vec!["Position"].into_iter()))
        }
    }
}

fn hide_subdivision_panel() -> Markup {
    html! {
        section.panel.fade {
            h3.underlined {
                (tr("toggle-subdivision-panel"))
            }
            p {
                (tr("toggle-subdivision-panel.info"))
            }
            div.cb-container.flex.no-stretch style="margin-bottom:10px" {
                i {(tr("toggle-subdivision-panel.option-toggle"))}
                input #show-subdivisions-checkbox type = "checkbox" checked="";
                span.checkmark {}
            }
        }
    }
}

struct StatsViewerRow(Vec<(String, &'static str)>);

fn standard_stats_viewer_rows() -> Vec<StatsViewerRow> {
    vec![
        StatsViewerRow(vec![(tr("statsviewer.rank"), "rank"), (tr("statsviewer.score"), "score")]),
        StatsViewerRow(vec![(tr("statsviewer.stats"), "stats"), (tr("statsviewer.hardest"), "hardest")]),
        StatsViewerRow(vec![(tr("statsviewer.completed"), "beaten")]),
        StatsViewerRow(vec![(tr("statsviewer.completed-main"), "main-beaten")]),
        StatsViewerRow(vec![(tr("statsviewer.completed-extended"), "extended-beaten")]),
        StatsViewerRow(vec![(tr("statsviewer.completed-legacy"), "legacy-beaten")]),
        StatsViewerRow(vec![
            (tr("statsviewer.created"), "created"),
            (tr("statsviewer.published"), "published"),
            (tr("statsviewer.verified"), "verified"),
        ]),
        StatsViewerRow(vec![(tr("statsviewer.progress"), "progress")]),
    ]
}

fn stats_viewer_html(nations: Option<&[Nationality]>, rows: Vec<StatsViewerRow>, is_nation_stats_viewer: bool) -> Markup {
    html! {
        section.panel.fade #statsviewer style="overflow:initial" {
            h2.underlined.pad {
                (tr("statsviewer"))
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
                                span class = "flag-icon" style={"background-image: url(/static/demonlist/images/flags/" (nation.iso_country_code.to_lowercase()) ".svg"} {}
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
                    @if is_nation_stats_viewer
                        { (tr("statsviewer-nation.welcome")) }
                    @else
                        { (tr("statsviewer-individual.welcome")) }

                }
                div.viewer-content {
                    div {
                        div.flex.col {
                            h3 #player-name style = "font-size:1.4em; overflow: hidden" {}
                            @for row in rows {
                                div.stats-container.flex.space {
                                    @for column in row.0 {
                                        span {
                                            b {
                                                (column.0)
                                            }
                                            br;
                                            span #(column.1) {}
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
