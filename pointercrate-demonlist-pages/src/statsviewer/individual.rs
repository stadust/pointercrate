use crate::statsviewer::stats_viewer_html;
use maud::{html, Markup};
use pointercrate_core_pages::{head::HeadLike, PageFragment};
use pointercrate_demonlist::nationality::Nationality;

#[derive(Debug)]
pub struct IndividualStatsViewer {
    pub nationalities_in_use: Vec<Nationality>,
}

impl From<IndividualStatsViewer> for PageFragment {
    fn from(stats_viewer: IndividualStatsViewer) -> Self {
        PageFragment::new(
            "Individual Stats Viewer",
            "The pointercrate individual stats viewer, a ranking of the worlds best Geometry Dash players. Now more local than ever, \
             allowing you to see who's the best in your state!",
        )
        .module("/static/demonlist/js/modules/statsviewer.js?v=4")
        .module("/static/demonlist/js/statsviewer/individual.js?v=4")
        .stylesheet("/static/demonlist/css/statsviewer.css")
        .stylesheet("/static/core/css/sidebar.css")
        .body(stats_viewer.body())
    }
}

impl IndividualStatsViewer {
    fn body(&self) -> Markup {
        html! {
            nav.flex.wrap.m-center.fade #statsviewers style="text-align: center;" {
                a.button.white.hover.no-shadow href="/demonlist/statsviewer/"{
                    b {"Individual"}
                }
                a.button.white.hover.no-shadow href="/demonlist/statsviewer/nations/" {
                    b {"Nations"}
                }
            }
            div #world-map-wrapper {
                object style="min-width:100%" #world-map data="/static/demonlist/images/world.svg" type="image/svg+xml" alt="World map showing the global demonlist score distribution" {}
            }
            div.flex.m-center.container {
                main.left {
                    (stats_viewer_html(Some(&self.nationalities_in_use), super::standard_stats_viewer_rows()))
                }
                aside.right {
                    (super::continent_panel())
                    (super::hide_subdivision_panel())
                    section.panel.fade style = "overflow: initial;" {
                        h3.underlined {
                            "Political Subdivision:"
                        }
                        p {
                            "For the "
                            span.tooltip {
                                "following countries"
                                span.tooltiptext.fade {
                                    "Argentina, Australia, Brazil, Canada, Chile, Colombia, Finland, France, Germany, Italy, Mexico, Netherlands, Norway, Peru, Poland, Russian Federation, South Korea, Spain, Ukraine, United Kingdom, United States"
                                }
                            }
                            " you can select a state/province from the dropdown below to focus the stats viewer to that state/province."
                        }
                        div.dropdown-menu.js-search #subdivision-dropdown data-default = "None" {
                            div{
                                input type="text" style = "color: #444446; font-weight: bold;";
                            }
                            div.menu {
                                ul {
                                    li.white.hover.underlined data-value = "None" {"None"}
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
