use crate::statsviewer::{stats_viewer_html, StatsViewerRow};
use maud::{html, Markup, PreEscaped};
use pointercrate_core_pages::{config, head::HeadLike, PageFragment};

pub fn nation_based_stats_viewer() -> PageFragment {
    PageFragment::new(
        "Nation Stats Viewer",
        "The pointercrate nation stats viewer, ranking how well each nations player's are doing in their quest to collectively complete \
         the entire demonlist!",
    )
    .module("/static/demonlist/js/modules/statsviewer.js")
    .module("/static/demonlist/js/statsviewer/nation.js")
    .stylesheet("/static/demonlist/css/statsviewer.css")
    .stylesheet("/static/core/css/sidebar.css")
    .body(nation_based_stats_viewer_html())
}

fn nation_based_stats_viewer_html() -> Markup {
    let mut rows = super::standard_stats_viewer_rows();

    rows[0].0.insert(1, ("Players", "players"));
    rows.push(StatsViewerRow(vec![("Unbeaten demons", "unbeaten")]));

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
                (stats_viewer_html(None, rows))
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
            }
        }
    }
}
