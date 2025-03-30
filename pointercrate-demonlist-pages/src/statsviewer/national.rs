use crate::statsviewer::{stats_viewer_html, StatsViewerRow};
use maud::{html, Markup};
use pointercrate_core::localization::tr;
use pointercrate_core_pages::{head::HeadLike, PageFragment};
use unic_langid::LanguageIdentifier;

pub fn nation_based_stats_viewer(lang: &'static LanguageIdentifier) -> PageFragment {
    PageFragment::new(
        "Nation Stats Viewer",
        "The pointercrate nation stats viewer, ranking how well each nation's players are doing in their quest to collectively complete \
         the entire demonlist!",
    )
    .module("/static/demonlist/js/modules/statsviewer.js")
    .module("/static/demonlist/js/statsviewer/nation.js")
    .stylesheet("/static/demonlist/css/statsviewer.css")
    .stylesheet("/static/core/css/sidebar.css")
    .body(nation_based_stats_viewer_html(lang))
}

fn nation_based_stats_viewer_html(lang: &'static LanguageIdentifier) -> Markup {
    let mut rows = super::standard_stats_viewer_rows(lang);

    rows[0].0.insert(1, (tr(lang, "statsviewer-nation.players"), "players"));
    rows.push(StatsViewerRow(vec![(tr(lang, "statsviewer-nation.unbeaten"), "unbeaten")]));

    html! {
        nav.flex.wrap.m-center.fade #statsviewers style="text-align: center; z-index: 1" {
            a.button.white.hover.no-shadow href="/demonlist/statsviewer/"{
                b {(tr(lang, "statsviewer-individual"))}
            }
            a.button.white.hover.no-shadow href="/demonlist/statsviewer/nations/" {
                b {(tr(lang, "statsviewer-nation"))}
            }
        }
        div #world-map-wrapper {
            object style="min-width:100%" #world-map data="/static/demonlist/images/world.svg" type="image/svg+xml" alt="World map showing the global demonlist score distribution" {}
        }
        div.flex.m-center.container {
            main.left {
                (stats_viewer_html(lang, None, rows, true))
            }
            aside.right {
                (super::demon_sorting_panel(lang))
                (super::continent_panel(lang))
            }
        }
    }
}
