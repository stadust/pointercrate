use crate::{
    extractor::auth::TokenAuth,
    permissions::Permissions,
    state::PointercrateState,
    view::{filtered_paginator, Page},
    ViewResult,
};
use actix_web::HttpResponse;
use actix_web_codegen::get;
use futures::StreamExt;
use maud::{html, Markup, PreEscaped};
use sqlx::query;
use std::collections::HashMap;

#[derive(Debug)]
struct StatsViewer {
    heatmap: HeatMap,
}

#[derive(Debug)]
struct HeatMap {
    map: HashMap<String, i64>,
}

impl HeatMap {
    pub fn compute_levels(&self, low_level: i64, high_level: i64) -> HashMap<&String, i64> {
        let sorted_values: Vec<i64> = {
            let mut values: Vec<i64> = self.map.values().map(|v| *v).collect();
            values.sort();
            values
        };

        let mut differences: Vec<(usize, i64)> = sorted_values.windows(2).map(|w| w[1] - w[0]).enumerate().collect();

        differences.sort();

        // search for local maxima in the data stream
        let mut division_points: Vec<usize> = differences
            .windows(3)
            .filter_map(|w| {
                if w[1].1 > w[0].1 && w[2].1 < w[1].1 {
                    Some(w[1].0 + 1)
                } else {
                    None
                }
            })
            .collect();

        if differences.len() > 1 && differences[0] > differences[1] {
            division_points.insert(0, 1);
        }

        let subdivisions = division_points.len() as i64 + 1;

        division_points.insert(0, 0);
        division_points.push(sorted_values.len());

        division_points.sort();

        let max_per_division: Vec<i64> = division_points.iter().skip(1).map(|&idx| sorted_values[idx - 1]).collect();
        let levels_per_subdivision = (high_level - low_level) / subdivisions;

        let mut level_map = HashMap::new();

        for (key, value) in &self.map {
            let rank = sorted_values.iter().position(|v| *v == *value).unwrap();
            let division = division_points.iter().rposition(|&idx| rank >= idx).unwrap();

            let base_level = low_level + ((high_level - low_level) / subdivisions) * (division as i64);

            let level = base_level + *value * levels_per_subdivision / max_per_division[division];

            level_map.insert(key, level);

            /*println!(
                "{} with total score {} at index {}, putting it at division {} with base level {} (highest in subdivision: {}). Thus its \
                 level is {}",
                key, value, rank, division, base_level, max_per_division[division], level
            );*/
        }

        level_map
    }
}

#[get("/demonlist/statsviewer/")]
pub async fn stats_viewer(TokenAuth(user): TokenAuth, state: PointercrateState) -> ViewResult<HttpResponse> {
    if !user.inner().has_permission(Permissions::Administrator) {
        user.inner().require_permissions(Permissions::ListHelper)?;
    }

    let mut connection = state.connection().await?;

    let mut heatmap = HashMap::new();

    let mut stream = query!(
        //r#"select iso_country_code as "iso_country_code!", sum(score)/count(*) as "cnt!" from players_with_score where iso_country_code is not null and score != 0 group by iso_country_code"#
        r#"select iso_country_code as "iso_country_code!", count(*) as "cnt!" from players_with_score where iso_country_code is not null and score != 0 group by iso_country_code"#
    )
    .fetch(&mut connection);

    while let Some(row) = stream.next().await {
        let row = row?;

        heatmap.insert(row.iso_country_code, row.cnt as i64);
    }

    Ok(HttpResponse::Ok().content_type("text/html; charset=utf-8").body(
        StatsViewer {
            heatmap: HeatMap { map: heatmap },
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
