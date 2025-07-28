use maud::{html, Markup};
use pointercrate_core::localization::tr;
use std::fmt::Display;

// FIXME: these should probably be turned into proper structs as well at some point

pub fn paginator(id: &str, endpoint: &str) -> Markup {
    html! {
        div.flex.col.paginator #(id) data-endpoint = (endpoint) {
            p.info-red.output {}
            p.info-green.output {}
            div style="min-height: 450px; position:relative; flex-grow:1" {
                ul.selection-list style = "position: absolute; top: 0px; bottom:0px; left: 0px; right:0px" {}
            }
            div.flex.no-stretch style = "font-variant: small-caps; font-weight: bolder; justify-content: space-around"{
                div.button.small.prev { (tr("paginator-previous")) }
                div.button.small.next { (tr("paginator-next")) }
            }
        }
    }
}

pub fn filtered_paginator(id: &str, endpoint: &str) -> Markup {
    html! {
        div.flex.col.paginator #(id) data-endpoint=(endpoint) {
            div.search.seperated.no-stretch {
                input placeholder = (tr("filtered-paginator-placeholder")) type = "text" style = "height: 1em";
            }
            p.info-red.output style = "margin: 5px 0px"{}
            div style="min-height: 400px; position:relative; flex-grow:1" {
                ul.selection-list style = "position: absolute; top: 0px; bottom:0px; left: 0px; right:0px" {}
            }
            div.flex.no-stretch style = "font-variant: small-caps; font-weight: bolder; justify-content: space-around"{
                div.button.small.prev { (tr("paginator-previous")) }
                div.button.small.next { (tr("paginator-next")) }
            }
        }
    }
}

pub fn dropdown(default_entry: &str, default_item: Markup, filter_items: impl Iterator<Item = Markup>) -> Markup {
    html! {
        div.dropdown-menu.js-search.no-stretch {
            div {
                input type="text" data-default=(default_entry) autocomplete="off" style = "font-weight: bold;";
            }
            div.menu {
                ul {
                    (default_item)
                    @for item in filter_items {
                        (item)
                    }
                }
            }
        }
    }
}

/// Items should be structured as `(<internal value>, <displayed value>)` where the internal value is consistent across languages
pub fn simple_dropdown<T1: Display, T2: Display>(
    dropdown_id: &str, default: Option<(T1, T2)>, items: impl Iterator<Item = (T1, T2)>,
) -> Markup {
    html! {
        div.dropdown-menu.js-search.no-stretch #(dropdown_id) {
            div {
                @match default {
                    Some(ref default) => {
                        input type="text" autocomplete="off" data-default=(default.0) style = "font-weight: bold;";
                    }
                    None => {
                        input type="text" autocomplete="off" style = "font-weight: bold;";
                    }
                }
            }

            div.menu {
                ul {
                    @if let Some(ref default) = default {
                        li.white.underlined.hover data-value=(default.0) data-display=(default.1) {
                            b {
                                (default.1)
                            }
                        }
                    }
                    @for item in items {
                        li.white.hover data-value=(item.0) data-display = (item.1)  {
                            b {
                                (item.1)
                            }
                        }
                    }
                }
            }
        }
    }
}

/// A version of the `trp!` marco that encapsulates some of the ugly details
/// of passing in rendered html as fluent placeholders. Particularly, it encapsulates
/// the required handling of maud::PreEscaped safely.
///
/// Essentially, this macro gives us the guarantee that as long as the input values dont use
/// maud::PreEscaped, then it will not be possible to inject unescaped data into the page using it.
#[macro_export]
macro_rules! trp_html {
    ($text_id:expr, $($key:literal = $value:expr),*)  => {
        maud::PreEscaped(pointercrate_core::trp!($text_id, $(
            $key = {let _: maud::PreEscaped<String> = $value; $value.into_string()}
        ),*))
    };
}