use self::heatmap::HeatMap;
use crate::{
    extractor::auth::TokenAuth,
    permissions::Permissions,
    state::PointercrateState,
    view::{filtered_paginator, Page},
    ViewResult,
};
use actix_web::HttpResponse;
use actix_web_codegen::get;
use maud::{html, Markup, PreEscaped};

mod heatmap;

#[derive(Debug)]
struct StatsViewer {
    heatmap: HeatMap,
}

#[get("/demonlist/statsviewer/")]
pub async fn stats_viewer(TokenAuth(user): TokenAuth, state: PointercrateState) -> ViewResult<HttpResponse> {
    if !user.inner().has_permission(Permissions::Administrator) {
        user.inner().require_permissions(Permissions::ListHelper)?;
    }

    let mut connection = state.connection().await?;

    Ok(HttpResponse::Ok().content_type("text/html; charset=utf-8").body(
        StatsViewer {
            heatmap: HeatMap::load_total_point_heatmap(&mut connection).await?,
        }
        .render()
        .0,
    ))
}

impl Page for StatsViewer {
    fn title(&self) -> String {
        "Stats Viewer".to_owned()
    }

    fn description(&self) -> String {
        "Stats Viewer".to_owned()
    }

    fn scripts(&self) -> Vec<&str> {
        vec!["js/statsviewer.js"]
    }

    fn stylesheets(&self) -> Vec<&str> {
        vec!["css/statsviewer.css", "css/sidebar.css"]
    }

    fn body(&self) -> Markup {
        let mut css_string = String::new();

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
        }

        html! {
            style {
                (PreEscaped(css_string))
            }
            div#world-map-wrapper {
                object#world-map data="/static2/images/world.svg" type="image/svg+xml" {}
            }
            div.flex.m-center.container {
                main.left {
                    (stats_viewer2())
                }
                aside.right {
                    div.panel.fade {
                        h3.underlined {
                            "Very important thing"
                        }
                        p {
                            "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Suspendisse lacinia nibh nec arcu bibendum, posuere maximus ante sodales. Duis bibendum dui vel velit gravida, semper laoreet magna efficitur. Vestibulum ac lectus vitae tortor bibendum placerat. Nunc varius, risus molestie fermentum molestie, ante ex fermentum ex, sed efficitur felis urna a urna. Suspendisse mattis finibus lectus. Fusce cursus nisl non facilisis laoreet. Donec lacus ipsum, rutrum at euismod et, fringilla nec tortor. Cras molestie sem sit amet tellus hendrerit, at feugiat purus ornare. Nunc consequat purus non condimentum efficitur. Etiam aliquet mollis ante et eleifend. Integer sagittis libero et erat ultricies volutpat. Nulla facilisi. "
                        }
                    }
                }
            }
        }
    }

    fn head(&self) -> Vec<Markup> {
        vec![html! {
            (PreEscaped(format!(r#"
                <link href="https://cdnjs.cloudflare.com/ajax/libs/flag-icon-css/3.4.3/css/flag-icon.min.css" rel="stylesheet">"#)))
        }]
    }
}

fn stats_viewer2() -> Markup {
    html! {
        section.panel.fade#statsviewer {
            h2.underlined.pad {
                "Stats Viewer - " span#current-nation {"International"}
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
                            div.stats-container.flex.space {
                                span {
                                    b {
                                        "List demons completed:"
                                    }
                                    br;
                                    span#amount-beaten {}
                                }
                                span {
                                    b {
                                        "Legacy demons completed:"
                                    }
                                    br;
                                    span#amount-legacy {}
                                }
                                span {
                                    b {
                                        "Demonlist score:"
                                    }
                                    br;
                                    span#score {}
                                }
                            }
                            div.stats-container.flex.space {
                                span {
                                    b {
                                        "Demonlist rank:"
                                    }
                                    br;
                                    span#rank {}
                                }
                                span {
                                    b {
                                        "Hardest demon:"
                                    }
                                    br;
                                    span#hardest {}
                                }
                            }
                            div.stats-container.flex.space {
                                span {
                                    b {
                                        "Demons completed:"
                                    }
                                    br;
                                    span#beaten {}
                                }
                            }
                            div.stats-container.flex.space {
                                span {
                                    b {
                                        "List demons created:"
                                    }
                                    br;
                                    span#created {}
                                }
                                span {
                                    b {
                                        "List demons published:"
                                    }
                                    br;
                                    span#published {}
                                }
                                span {
                                    b {
                                        "List demons verified:"
                                    }
                                    br;
                                    span#verified {}
                                }
                            }
                            div.stats-container.flex.space {
                                span {
                                    b {
                                        "Progress on:"
                                    }
                                    br;
                                    span#progress {}
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
