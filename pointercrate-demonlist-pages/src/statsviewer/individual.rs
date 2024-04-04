use crate::statsviewer::stats_viewer_html;
use maud::{html, Markup, PreEscaped};
use pointercrate_core_pages::{config, head::HeadLike, PageFragment};
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
        .module("/static/demonlist/js/modules/statsviewer.js")
        .module("/static/demonlist/js/statsviewer/individual.js")
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
                object #world-map data="/static/demonlist/images/world.svg" type="image/svg+xml" {}
            }
            div.flex.m-center.container {
                main.left {
                    @if let Some(publisher_id) = config::adsense_publisher_id() {
                        section.panel.fade style = "padding: 0px; height: 90px"{
                            (PreEscaped(format!(r#"
                            <script async src="https://pagead2.googlesyndication.com/pagead/js/adsbygoogle.js?client={0}"
                                 crossorigin="anonymous"></script>
                            <!-- Statsviewer Banner Ad -->
                            <ins class="adsbygoogle"
                                 style="display:inline-block;width:728px;height:90px"
                                 data-ad-client="{}"
                                 data-ad-slot="5855948132"></ins>
                            <script>
                                 (adsbygoogle = window.adsbygoogle || []).push({{}});
                            </script>
                            "#, publisher_id)))
                        }
                    }
                    (stats_viewer_html(Some(&self.nationalities_in_use), super::standard_stats_viewer_rows()))
                }
                aside.right {
                    (super::continent_panel())
                    @if let Some(publisher_id) = config::adsense_publisher_id() {
                        section.panel.fade.js-scroll-anim data-anim = "fade" style = "order: 1; padding: 0px; border: 0" {
                            (PreEscaped(format!(r#"
                            <script async src="https://pagead2.googlesyndication.com/pagead/js/adsbygoogle.js?client={0}"
                                 crossorigin="anonymous"></script>
                            <!-- Statsviewer Sidebar Ad -->
                            <ins class="adsbygoogle"
                                 style="display:block"
                                 data-ad-client="{0}"
                                 data-ad-slot="2211027222"
                                 data-ad-format="auto"
                                 data-full-width-responsive="true"></ins>
                            <script>
                                 (adsbygoogle = window.adsbygoogle || []).push({{}});
                            </script>
                            "#, publisher_id)))
                        }
                    }
                    (super::hide_subdivision_panel())
                    section.panel.fade style = "overflow: initial;" {
                        h3.underlined {
                            "Political Subdivision:"
                        }
                        p {
                            "For the " i {"United States of America"} ", " i {"The United Kingdom of Great Britain and Northern Ireland"} ", " i{"Australia"} " and " i{"Canada"} " you can select a state/province from the dropdown below to focus the stats viewer to that state/province."
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
