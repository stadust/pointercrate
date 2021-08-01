use super::stats_viewer_html;
use crate::{config, model::nationality::Nationality, state::PointercrateState, view::Page, ViewResult};
use actix_web::HttpResponse;
use actix_web_codegen::get;
use maud::{html, Markup, PreEscaped};

#[derive(Debug)]
struct IndividualStatsViewer {
    //heatmap: HeatMap,
    nationalities_in_use: Vec<Nationality>,
}

#[get("/demonlist/statsviewer/")]
pub async fn stats_viewer(state: PointercrateState) -> ViewResult<HttpResponse> {
    let mut connection = state.connection().await?;

    Ok(HttpResponse::Ok().content_type("text/html; charset=utf-8").body(
        IndividualStatsViewer {
            //heatmap: HeatMap::load_total_point_heatmap(&mut connection).await?,
            nationalities_in_use: Nationality::used(&mut *connection).await?,
        }
        .render()
        .0,
    ))
}

impl Page for IndividualStatsViewer {
    fn title(&self) -> String {
        "Individual Stats Viewer".to_owned()
    }

    fn description(&self) -> String {
        "The pointercrate individual stats viewer, a ranking of the worlds best Geometry Dash players. Now more local than ever, allowing \
         you to see who's the best in your state!"
            .to_owned()
    }

    fn scripts(&self) -> Vec<&str> {
        vec!["js/statsviewer.js", "js/statsviewer/individual.js"]
    }

    fn stylesheets(&self) -> Vec<&str> {
        vec!["css/statsviewer.css", "css/sidebar.css"]
    }

    fn body(&self) -> Markup {
        /*let mut css_string = String::new();

        for (nationality, level) in self.heatmap.compute_levels(0, 100) {
            let level = level / 2 + 10;

            // heat map by gradient from dadce0 to 0881c6
            css_string.push_str(&format!(
                "#{} path {{ fill: rgb({}, {}, {}); }}",
                nationality.to_lowercase(),
                0xda + (0x08 - 0xda) * level / 100,
                0xdc + (0x81 - 0xdc) * level / 100,
                0xe0 + (0xc6 - 0xe0) * level / 100,
            ))
        }*/

        html! {
            nav.flex.wrap.m-center.fade#statsviewers style="text-align: center;" {
                a.button.white.hover.no-shadow href="/demonlist/statsviewer/"{
                    b {"Individual"}
                }
                a.button.white.hover.no-shadow href="/demonlist/statsviewer/nations/" {
                    b {"Nations"}
                }
            }
            /*style {
                (PreEscaped(css_string))
            }*/
            div#world-map-wrapper {
                object#world-map data="/static2/images/world.svg" type="image/svg+xml" {}
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

    fn head(&self) -> Vec<Markup> {
        vec![]
    }
}
