//! Module containing various UI components that are used across a variety of demonlist pages

use maud::{html, Markup, Render};
use pointercrate_core::{localization::tr, trp};
use pointercrate_demonlist::demon::Demon;
use pointercrate_demonlist::player::DatabasePlayer;

pub mod team;
pub mod time_machine;

pub fn demon_dropdown(dropdown_id: &str, demons: &[Demon], initial_demon: Option<usize>) -> Markup {
    let initial_demon_name: &str = match initial_demon {
        Some(position) => demons.get(position - 1).map_or("", |demon| &demon.base.name),
        None => "",
    };

    html! {
        div.dropdown-menu.js-search #(dropdown_id) {
            div {
                input type = "text" name = "demon" required="" autocomplete="off" value=(initial_demon_name);
            }
            div.menu {
               ul {
                    @for demon in demons {
                        li.white.hover data-value = (demon.base.id) data-display = (demon.base.name) {b{"#"(demon.base.position) " - " (demon.base.name)} br; { (trp!("demon-listed.publisher", "publisher" = demon.publisher.name)) }}
                    }
                }
            }
        }
    }
}

pub fn player_selection_dropdown(
    dropdown_id: &str, endpoint: &str, field: &str, form_field: &str, initial_player: &Option<DatabasePlayer>,
) -> Markup {
    let initial_player_name: &str = match initial_player {
        Some(player) => &player.name,
        None => "",
    };
    html! {
        div.dropdown-menu #(dropdown_id) data-endpoint = (endpoint) data-field = (field) {
            div {
                input type = "text" name = (form_field) required="" autocomplete="off" placeholder = (tr("record-submission.holder-input-placeholder")) value = (initial_player_name);
            }
            div.menu {
                // dynamically populated once the user starts typing
                ul {}
            }
        }
    }
}

pub fn player_selection_dialog(
    dialog_id: &str, dropdown_id: &str, headline: &str, description: &str, button_text: &str, form_field: &str,
    initial_player: &Option<DatabasePlayer>,
) -> Markup {
    html! {
        div.overlay.closable {
            div.dialog #(dialog_id) style="overflow: initial;" {
                span.plus.cross.hover {}
                h2.underlined.pad {
                    (headline)
                }
                form.flex.col novalidate {
                    p {
                        (description)
                    }
                    span.form-input.flex.col data-type = "dropdown" {
                        (player_selection_dropdown(dropdown_id, "/api/v1/players/", "name", form_field, initial_player))
                        p.error {}
                    }
                    input.button.blue.hover type = "submit" style = "margin: 15px auto 0px;" value = (button_text);
                }
            }
        }
    }
}

pub struct P<'a>(pub &'a DatabasePlayer, pub Option<&'static str>);

impl Render for P<'_> {
    fn render(&self) -> Markup {
        if let Some(id) = self.1 {
            html! {
                a.underdotted #(id) href = {"/demonlist/statsviewer?player="(self.0.id)} data-id = (self.0.id) target = "_blank" {
                    (self.0.name)
                }
            }
        } else {
            html! {
                a.underdotted href = {"/demonlist/statsviewer?player="(self.0.id)} data-id = (self.0.id) target = "_blank" {
                    (self.0.name)
                }
            }
        }
    }
}
