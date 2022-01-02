use maud::{html, Markup};
use std::fmt::Display;

// FIXME: these should probably be turned into proper structs as well at some point

pub fn paginator(id: &str, endpoint: &str) -> Markup {
    html! {
        div.flex.col.paginator#(id) data-endpoint = (endpoint) {
            p.info-red.output {}
            div style="min-height: 450px; position:relative; flex-grow:1" {
                ul.selection-list style = "position: absolute; top: 0px; bottom:0px; left: 0px; right:0px" {}
            }
            div.flex.no-stretch style = "font-variant: small-caps; font-weight: bolder; justify-content: space-around"{
                div.button.small.prev { "Previous" }
                div.button.small.next { "Next" }
            }
        }
    }
}

pub fn filtered_paginator(id: &str, endpoint: &str) -> Markup {
    html! {
        div.flex.col.paginator#(id) data-endpoint=(endpoint) {
            div.search.seperated.no-stretch {
                input placeholder = "Enter to search..." type = "text" style = "height: 1em";
            }
            p.info-red.output style = "margin: 5px 0px"{}
            div style="min-height: 400px; position:relative; flex-grow:1" {
                ul.selection-list style = "position: absolute; top: 0px; bottom:0px; left: 0px; right:0px" {}
            }
            div.flex.no-stretch style = "font-variant: small-caps; font-weight: bolder; justify-content: space-around"{
                div.button.small.prev { "Previous" }
                div.button.small.next { "Next" }
            }
        }
    }
}

pub fn dropdown(default_entry: &str, default_item: Markup, filter_items: impl Iterator<Item = Markup>) -> Markup {
    html! {
        div.dropdown-menu.js-search.no-stretch {
            div {
                input type="text" data-default=(default_entry) autocomplete="off" style = "color: #444446; font-weight: bold;";
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

pub fn simple_dropdown<T1: Display>(dropdown_id: &str, default: Option<T1>, items: impl Iterator<Item = T1>) -> Markup {
    html! {
        div.dropdown-menu.js-search.no-stretch#(dropdown_id) {
            div {
                @match default {
                    Some(ref default) => {
                        input type="text" autocomplete="off" data-default=(default) style = "color: #444446; font-weight: bold;";
                    }
                    None => {
                        input type="text" autocomplete="off" style = "color: #444446; font-weight: bold;";
                    }
                }
            }

            div.menu {
                ul {
                    @if let Some(ref default) = default {
                        li.white.underlined.hover data-value=(default) data-display=(default) {
                            b {
                                (default)
                            }
                        }
                    }
                    @for item in items {
                        li.white.hover data-value=(item) data-display = (item)  {
                            b {
                                (item)
                            }
                        }
                    }
                }
            }
        }
    }
}
