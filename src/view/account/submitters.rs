use crate::view::paginator;
use maud::{html, Markup};

pub(super) fn page() -> Markup {
    html! {
        div.m-center.flex.tab-content.container data-tab-id = "6"{
            div.left {
                div.panel.fade {
                    h2.underlined.pad {
                        "Submitter Manager"
                    }
                    div.flex.viewer {
                        (paginator("submitter-pagination", "/api/v1/submitters/"))
                        p.viewer-welcome {
                            "Click on a submitter on the left to get started!"
                        }
                    }
                }
            }
            div.right {

            }
        }
    }
}
