use crate::statsviewer::{stats_viewer_html, StatsViewerRow};
use maud::{html, Markup, PreEscaped};
use pointercrate_core_pages::{config, PageFragment, Script};

#[derive(Debug)]
pub struct NationBasedStatsViewer;

impl PageFragment for NationBasedStatsViewer {
    fn title(&self) -> String {
        "Nation Stats Viewer".to_owned()
    }

    fn description(&self) -> String {
        "The pointercrate nation stats viewer, ranking how well each nations player's are doing in their quest to collectively complete \
         the entire demonlist!"
            .to_owned()
    }

    fn additional_scripts(&self) -> Vec<Script> {
        vec![
            Script::module("/static/js/statsviewer.js"),
            Script::module("/static/js/statsviewer/nation.js"),
        ]
    }

    fn additional_stylesheets(&self) -> Vec<String> {
        vec!["/static/css/statsviewer.css".to_string(), "/static/css/sidebar.css".to_string()]
    }

    fn head_fragment(&self) -> Markup {
        html! {}
    }

    fn body_fragment(&self) -> Markup {
        let mut rows = super::standard_stats_viewer_rows();

        rows[0].0.insert(1, ("Players", "players"));
        rows.push(StatsViewerRow(vec![("Unbeaten demons", "unbeaten")]));

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
                    (stats_viewer_html(None, rows))
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
                }
            }
        }
    }
}
