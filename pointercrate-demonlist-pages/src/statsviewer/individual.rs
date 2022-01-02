use crate::statsviewer::stats_viewer_html;
use maud::{html, Markup, PreEscaped};
use pointercrate_core_pages::{config, PageFragment, Script};
use pointercrate_demonlist::nationality::Nationality;

#[derive(Debug)]
pub struct IndividualStatsViewer {
    pub nationalities_in_use: Vec<Nationality>,
}

impl PageFragment for IndividualStatsViewer {
    fn title(&self) -> String {
        "Individual Stats Viewer".to_owned()
    }

    fn description(&self) -> String {
        "The pointercrate individual stats viewer, a ranking of the worlds best Geometry Dash players. Now more local than ever, allowing \
         you to see who's the best in your state!"
            .to_owned()
    }

    fn additional_scripts(&self) -> Vec<Script> {
        vec![
            Script::module("/static/js/statsviewer.js"),
            Script::module("/static/js/statsviewer/individual.js"),
        ]
    }

    fn additional_stylesheets(&self) -> Vec<String> {
        vec!["/static/css/statsviewer.css".to_string(), "/static/css/sidebar.css".to_string()]
    }

    fn head_fragment(&self) -> Markup {
        html! {}
    }

    fn body_fragment(&self) -> Markup {
        html! {
            nav.flex.wrap.m-center.fade#statsviewers style="text-align: center;" {
                a.button.white.hover.no-shadow href="/demonlist/statsviewer/"{
                    b {"Individual"}
                }
                a.button.white.hover.no-shadow href="/demonlist/statsviewer/nations/" {
                    b {"Nations"}
                }
            }
            div#world-map-wrapper {
                object#world-map data="/static/images/world.svg" type="image/svg+xml" {}
            }
            div.flex.m-center.container {
                main.left {
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
                        "#, config::adsense_publisher_id())))
                    }
                    (stats_viewer_html(Some(&self.nationalities_in_use), super::standard_stats_viewer_rows()))
                }
                aside.right {
                    (super::continent_panel())
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
                        "#, config::adsense_publisher_id())))
                    }
                    (super::hide_subdivision_panel())
                    section.panel.fade style = "overflow: initial;" {
                        h3.underlined {
                            "Political Subdivision:"
                        }
                        p {
                            "For the " i {"United States of America"} ", " i {"The United Kingdom of Great Britain and Northern Ireland"} ", " i{"Australia"} " and " i{"Canada"} " you can select a state/province from the dropdown below to focus the stats viewer to that state/province."
                        }
                        div.dropdown-menu.js-search#subdivision-dropdown data-default = "None" {
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
