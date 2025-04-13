use maud::{html, Markup};

pub fn locale_selection_item(flag: &str, code: &str) -> Markup {
    // todo: set title attribute of flag to match country name
    html! {
        span data-flag = (flag) data-lang = (code) {
            span.flag-icon style = (format!(r#"background-image: url("/static/demonlist/images/flags/{}.svg");"#, flag)) {}
            span style = "margin-left: 10px" { (code.to_uppercase()) }
        }
    }
}
