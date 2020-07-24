use crate::view::filtered_paginator;
use maud::{html, Markup};

pub(super) fn page() -> Markup {
    html! {
        div.m-center.flex.tab-content.container data-tab-id = "5"{
            div.left {
                div.panel.fade {
                    h2.underlined.pad {
                        "Demon Manager"
                    }
                    div.flex.viewer {
                        (filtered_paginator("demon-pagination", "/api/v1/demons/"))
                        p.viewer-welcome {
                            "Click on a demon on the left to get started!"
                        }
                    }
                }
            }
            div.right {

            }
        }
    }
}
