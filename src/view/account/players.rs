use crate::view::filtered_paginator;
use maud::{html, Markup};

pub(super) fn page() -> Markup {
    html! {
        div.m-center.flex.tab-content.container data-tab-id = "4"{
            div.left {
                div.panel.fade {
                    h2.underlined.pad {
                        "Player Manager"
                    }
                    div.flex.viewer {
                        (filtered_paginator("player-pagination", "/api/v1/players/"))
                        p.viewer-welcome {
                            "Click on a player on the left to get started!"
                        }
                    }
                }
            }
            div.right {

            }
        }
    }
}
