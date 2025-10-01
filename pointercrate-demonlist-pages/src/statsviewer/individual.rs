use crate::statsviewer::stats_viewer_html;
use maud::{html, Markup, PreEscaped};
use pointercrate_core::localization::tr;
use pointercrate_core_pages::{head::HeadLike, trp_html, PageFragment};
use pointercrate_demonlist::{list::List, nationality::Nationality};

#[derive(Debug)]
pub struct IndividualStatsViewer {
    pub nationalities_in_use: Vec<Nationality>,
    pub list: List,
}

impl From<IndividualStatsViewer> for PageFragment {
    fn from(stats_viewer: IndividualStatsViewer) -> Self {
        PageFragment::new(
            "Individual Stats Viewer",
            "The pointercrate individual stats viewer, a ranking of the worlds best Geometry Dash players. Now more local than ever, \
             allowing you to see who's the best in your state!",
        )
        .module("/static/demonlist/js/modules/statsviewer.js")
        .module("/static/demonlist/js/statsviewer/individual.js")
        .stylesheet("/static/demonlist/css/statsviewer.css")
        .stylesheet("/static/core/css/sidebar.css")
        .head(html! {
            (PreEscaped(format!(r#"<link href="/{0}/statsviewer/heatmap.css" rel="stylesheet" type="text/css"/>"#, &stats_viewer.list.as_str())))
        })
        .body(stats_viewer.body())
    }
}

impl IndividualStatsViewer {
    fn body(&self) -> Markup {
        html! {
            nav.flex.wrap.m-center.fade #statsviewers style="text-align: center; z-index: 1" {
                a.button.white.hover.no-shadow href=(format!("/{}/statsviewer/", &self.list.as_str())){
                    b {(tr("statsviewer-individual"))}
                }
                a.button.white.hover.no-shadow href=(format!("/{}/statsviewer/nations/", &self.list.as_str())) {
                    b {(tr("statsviewer-nation"))}
                }
            }
            (super::world_map())
            div.flex.m-center.container {
                main.left {
                    (stats_viewer_html(Some(&self.nationalities_in_use), super::standard_stats_viewer_rows(&self.list), false))
                }
                aside.right {
                    (super::demon_sorting_panel())
                    (super::continent_panel())
                    (super::hide_subdivision_panel())
                    section.panel.fade style = "overflow: initial;" {
                        h3.underlined {
                            (tr("subdivision-panel"))
                        }
                        p {
                            (trp_html!(
                                "subdivision-panel.info",
                                "countries" =
                                html! {
                                    span.tooltip {
                                        (tr("subdivision-panel.info-countries"))

                                        span.tooltiptext.fade {
                                            r#"Argentina, Australia, Brazil, Canada, Chile, Colombia, Finland, France, Germany, Italy, Mexico, Netherlands, Norway, Peru, Poland, Russian Federation, South Korea, Spain, Ukraine, United Kingdom, United States"#
                                        }
                                    }
                                }
                            ))
                        }
                        div.dropdown-menu.js-search #subdivision-dropdown data-default = "None" {
                            div{
                                input type="text" style = "cfont-weight: bold;";
                            }
                            div.menu {
                                ul {
                                    li.white.hover.underlined data-value = "None" {(tr("subdivision-panel.option-none"))}
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
