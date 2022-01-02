//! Module containing various UI components that are used across a variety of demonlist pages

use maud::{html, Markup};
use pointercrate_core_pages::util::filtered_paginator;
use pointercrate_demonlist::demon::Demon;

pub mod submitter;
pub mod team;
pub mod time_machine;

pub fn demon_dropdown<'a>(dropdown_id: &str, demons: impl Iterator<Item = &'a Demon>) -> Markup {
    html! {
        div.dropdown-menu.js-search#(dropdown_id) {
            div {
                input type = "text" name = "demon" required="" autocomplete="off";
            }
            div.menu {
               ul {
                    @for demon in demons {
                        li.white.hover data-value = (demon.base.id) data-display = (demon.base.name) {b{"#"(demon.base.position) " - " (demon.base.name)} br; {"by "(demon.publisher.name)}}
                    }
                }
            }
        }
    }
}

pub fn player_selection_dialog(dialog_id: &str, headline: &str, description: &str, button_text: &str) -> Markup {
    html! {
        div.overlay.closable {
            div.dialog#(dialog_id) {
                span.plus.cross.hover {}
                h2.underlined.pad {
                    (headline)
                }
                div.flex.viewer {
                    (filtered_paginator(&format!("{}-pagination", dialog_id), "/api/v1/players/"))
                    div {
                        p {
                            (description)
                        }
                        form.flex.col novalidate = "" {
                            p.info-red.output {}
                            p.info-green.output {}
                            span.form-input#{(dialog_id)"-input"} {
                                label for = "player" {"Player name:"}
                                input name = "player" type="text" required = "";
                                p.error {}
                            }
                            input.button.blue.hover type = "submit" style = "margin: 15px auto 0px;" value = (button_text);
                        }
                    }
                }
            }
        }
    }
}
