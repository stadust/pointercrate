use crate::{model::nationality::Nationality, view::filtered_paginator};
use maud::{html, Markup, PreEscaped};

pub(super) fn page(nationalities: &[Nationality]) -> Markup {
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
                        div.viewer-content {
                            div.flex.col{
                                h3 style = "font-size:1.1em; margin: 10px 0" {
                                    "Player #"
                                    i#player-player-id {}
                                    " - "
                                    i.fa.fa-pencil.clickable#player-name-pen aria-hidden = "true" {} (PreEscaped("&nbsp;")) i#player-player-name {}
                                }
                                p {
                                    "Welcome to the player manager. Here you can ban or unban players. Banning a player will delete all records of theirs which are in the submitted or under consideration state. All approved records will instead be set to rejected."
                                }
                                p.info-red.output style = "margin: 10px" {}
                                p.info-green.output style = "margin: 10px" {}
                                div.stats-container.flex.space {
                                    span {
                                        b {
                                            "Banned:"
                                        }
                                        br;
                                        div.dropdown-menu.js-search#edit-player-banned style = "max-width: 50px"{
                                            input type="text" style = "color: #444446; font-weight: bold;";
                                            div.menu {
                                                ul {
                                                    li.white.hover data-value="true" {"yes"}
                                                    li.white.hover data-value="false" {"no"}
                                                }
                                            }
                                        }
                                    }
                                    span {
                                        b {
                                            "Nationality:"
                                        }
                                        br;
                                        div.dropdown-menu.js-search#edit-player-nationality data-default = "None" {
                                            input type="text" style = "color: #444446; font-weight: bold;";
                                            div.menu {
                                                ul {
                                                    li.white.hover.underlined data-value = "None" {"None"}
                                                    @for nation in nationalities {
                                                        li.white.hover data-value = {(nation.iso_country_code)} data-display = {(nation.nation)} {
                                                            span class = {"flag-icon flag-icon-" (nation.iso_country_code.to_lowercase())} {}
                                                            (PreEscaped("&nbsp;"))
                                                            b {(nation.iso_country_code)}
                                                            br;
                                                            span style = "font-size: 90%; font-style: italic" {(nation.nation)}
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                span.button.blue.hover#player-list-records style = "margin: 15px auto 0px" {"Show records in record manager"};
                            }
                        }
                    }
                }
                div style="height: 50px" {} // to make sure that the footer doesnt float. if it floats, the user page is the only one without a scrollbar at the right, which causes jumpyness when switching tabs.
            }
            div.right {
                (player_selector())
            }
            (change_name_dialog())
        }
    }
}

fn player_selector() -> Markup {
    html! {
        div.panel.fade {
            h2.underlined.pad {
                "Search player by ID"
            }
            p {
                "Players can be uniquely identified by ID. Entering a players's ID below will select it on the left (provided the player exists)"
            }
            form.flex.col#player-search-by-player-id-form novalidate = "" {
                p.info-red.output {}
                span.form-input#search-player-id {
                    label for = "id" {"Player ID:"}
                    input required = "" type = "number" name = "id" min = "0" style="width:93%";
                    p.error {}
                }
                input.button.blue.hover type = "submit" style = "margin: 15px auto 0px;" value="Find by ID";
            }
        }
    }
}

fn change_name_dialog() -> Markup {
    html! {
        div.overlay.closable {
            div.dialog#player-name-dialog {
                span.plus.cross.hover {}
                h2.underlined.pad {
                    "Change player name:"
                }
                p style = "max-width: 400px"{
                    "Change the name of this player. This will update their name on every one of their records. If a player with the new name already exists, the player objects will be merged, with the new object receiving the ID of the player you are currently editing. In this case, the record lists of the players are merged and their creator/verifier/publisher information is updated. Internally, each record is moved to to the new player, an on conflicts the same rules apply as when editing a record's holder."
                }
                form.flex.col novalidate = "" {
                    p.info-red.output {}
                    p.info-green.output {}
                    span.form-input#player-name-edit {
                        label for = "name" {"Name:"}
                        input name = "name" type = "text" required = "";
                        p.error {}
                    }
                    input.button.blue.hover type = "submit" style = "margin: 15px auto 0px;" value = "Edit";
                }
            }
        }
    }
}
