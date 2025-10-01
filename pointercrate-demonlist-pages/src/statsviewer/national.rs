use crate::statsviewer::{stats_viewer_html, StatsViewerRow};
use maud::{html, Markup, PreEscaped};
use pointercrate_core::localization::tr;
use pointercrate_core_pages::{head::HeadLike, PageFragment};
use pointercrate_demonlist::list::List;

pub fn nation_based_stats_viewer(list: &List) -> PageFragment {
    PageFragment::new(
        "Nation Stats Viewer",
        "The pointercrate nation stats viewer, ranking how well each nation's players are doing in their quest to collectively complete \
         the entire demonlist!",
    )
    .module("/static/demonlist/js/modules/statsviewer.js")
    .module("/static/demonlist/js/statsviewer/nation.js")
    .stylesheet("/static/demonlist/css/statsviewer.css")
    .stylesheet("/static/core/css/sidebar.css")
    .head(html! {
        (PreEscaped(format!(r#"<link href="/{0}/statsviewer/heatmap.css" rel="stylesheet" type="text/css"/>"#, &list.as_str())))
    })
    .body(nation_based_stats_viewer_html(list))
}

fn nation_based_stats_viewer_html(list: &List) -> Markup {
    let mut rows = super::standard_stats_viewer_rows(list);

    rows[0].0.insert(1, (tr("statsviewer-nation.players"), "players"));
    rows.push(StatsViewerRow(vec![(tr("statsviewer-nation.unbeaten"), "unbeaten")]));

    html! {
        nav.flex.wrap.m-center.fade #statsviewers style="text-align: center; z-index: 1" {
            a.button.white.hover.no-shadow href=(format!("/{}/statsviewer/", list.as_str())) {
                b {(tr("statsviewer-individual"))}
            }
            a.button.white.hover.no-shadow href=(format!("/{}/statsviewer/nations/", list.as_str())) {
                b {(tr("statsviewer-nation"))}
            }
        }
        (super::world_map())
        div.flex.m-center.container {
            main.left {
                (stats_viewer_html(None, rows, true))
            }
            aside.right {
                (super::demon_sorting_panel())
                (super::continent_panel())
            }
        }
    }
}
