use maud::{html, Markup};

pub(super) fn page() -> Markup {
    html! {
        div.m-center.flex.tab-content.tab-content-active.container data-tab-id = "6"{
            div.left {

            }
            div.right {

            }
        }
    }
}
