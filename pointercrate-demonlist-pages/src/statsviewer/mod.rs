use maud::{html, Markup, PreEscaped};
use pointercrate_core::localization::tr;
use pointercrate_core_pages::util::{dropdown, filtered_paginator, simple_dropdown};
use pointercrate_demonlist::nationality::Nationality;
use unic_langid::LanguageIdentifier;

pub mod individual;
pub mod national;

pub(crate) fn stats_viewer_panel(lang: &'static LanguageIdentifier) -> Markup {
    html! {
        section #stats.panel.fade.js-scroll-anim data-anim = "fade" {
            div.underlined {
                h2 {
                    (tr(lang, "statsviewer-panel"))
                }
            }
            p {
                (tr(lang, "statsviewer-panel.info"))
            }
            a.blue.hover.button #show-stats-viewer href = "/demonlist/statsviewer/ "{
                (tr(lang, "statsviewer-panel.button"))
            }
        }
    }
}

fn continent_panel(lang: &'static LanguageIdentifier) -> Markup {
    html! {
        section.panel.fade style="overflow:initial"{
            h3.underlined {
                (tr(lang, "continent-panel"))
            }
            p {
                (tr(lang, "continent-panel.info"))
            }
            (simple_dropdown("continent-dropdown", Some("All"), vec!["Asia", "Europe", "Australia", "Africa", "North America", "South America", "Central America"].into_iter()))
        }
    }
}

fn demon_sorting_panel(lang: &'static LanguageIdentifier) -> Markup {
    html! {
        section.panel.fade style="overflow:initial" {
            h3.underlined {
                (tr(lang, "demon-sorting-panel"))
            }
            p {
                (tr(lang, "demon-sorting-panel.info"))
            }
            (simple_dropdown("demon-sorting-mode-dropdown", Some("Alphabetical"), vec!["Position"].into_iter()))
        }
    }
}

fn hide_subdivision_panel(lang: &'static LanguageIdentifier) -> Markup {
    html! {
        section.panel.fade {
            h3.underlined {
                (tr(lang, "toggle-subdivision-panel"))
            }
            p {
                (tr(lang, "toggle-subdivision-panel.info"))
            }
            div.cb-container.flex.no-stretch style="margin-bottom:10px" {
                i {(tr(lang, "toggle-subdivision-panel.option-toggle"))}
                input #show-subdivisions-checkbox type = "checkbox" checked="";
                span.checkmark {}
            }
        }
    }
}

struct StatsViewerRow(Vec<(String, &'static str)>);

fn standard_stats_viewer_rows(lang: &'static LanguageIdentifier) -> Vec<StatsViewerRow> {
    vec![
        StatsViewerRow(vec![
            (tr(lang, "statsviewer.rank"), "rank"),
            (tr(lang, "statsviewer.score"), "score"),
        ]),
        StatsViewerRow(vec![
            (tr(lang, "statsviewer.stats"), "stats"),
            (tr(lang, "statsviewer.hardest"), "hardest"),
        ]),
        StatsViewerRow(vec![(tr(lang, "statsviewer.completed"), "beaten")]),
        StatsViewerRow(vec![(tr(lang, "statsviewer.completed-main"), "main-beaten")]),
        StatsViewerRow(vec![(tr(lang, "statsviewer.completed-extended"), "extended-beaten")]),
        StatsViewerRow(vec![(tr(lang, "statsviewer.completed-legacy"), "legacy-beaten")]),
        StatsViewerRow(vec![
            (tr(lang, "statsviewer.created"), "created"),
            (tr(lang, "statsviewer.published"), "published"),
            (tr(lang, "statsviewer.verified"), "verified"),
        ]),
        StatsViewerRow(vec![(tr(lang, "statsviewer.progress"), "progress")]),
    ]
}

fn stats_viewer_html(
    lang: &'static LanguageIdentifier, nations: Option<&[Nationality]>, rows: Vec<StatsViewerRow>, is_nation_stats_viewer: bool,
) -> Markup {
    html! {
        section.panel.fade #statsviewer style="overflow:initial" {
            h2.underlined.pad {
                (tr(lang, "statsviewer"))
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
                        { (tr(lang, "statsviewer-nation.welcome")) }
                    @else
                        { (tr(lang, "statsviewer-individual.welcome")) }

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
