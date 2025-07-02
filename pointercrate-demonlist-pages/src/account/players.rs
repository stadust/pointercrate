use maud::{html, Markup, PreEscaped};
use pointercrate_core::{error::PointercrateError, localization::tr, permission::PermissionsManager};
use pointercrate_core_pages::{error::ErrorFragment, util::filtered_paginator};
use pointercrate_demonlist::{nationality::Nationality, LIST_MODERATOR};
use pointercrate_user::auth::{AuthenticatedUser, NonMutating};
use pointercrate_user_pages::account::AccountPageTab;
use sqlx::PgConnection;

pub struct PlayersPage;

#[async_trait::async_trait]
impl AccountPageTab for PlayersPage {
    fn should_display_for(&self, permissions_we_have: u16, permissions: &PermissionsManager) -> bool {
        permissions.require_permission(permissions_we_have, LIST_MODERATOR).is_ok()
    }

    fn initialization_script(&self) -> String {
        "/static/demonlist/js/account/player.js".into()
    }

    fn tab_id(&self) -> u8 {
        4
    }

    fn tab(&self) -> Markup {
        html! {
            b {
                (tr("players"))
            }
            (PreEscaped("&nbsp;&nbsp;"))
            i class = "fa fa-beer fa-2x" aria-hidden="true" {}
        }
    }

    async fn content(
        &self, _user: &AuthenticatedUser<NonMutating>, _permissions: &PermissionsManager, connection: &mut PgConnection,
    ) -> Markup {
        let nationalities = match Nationality::all(connection).await {
            Ok(nationalities) => nationalities,
            Err(err) => {
                return ErrorFragment {
                    status: err.status_code(),
                    reason: "Internal Server Error".to_string(),
                    message: err.to_string(),
                }
                .body()
            },
        };

        html! {
            div.left {
                div.panel.fade style = "overflow: initial"{
                    h2.underlined.pad {
                        (tr("player-manager"))
                    }
                    div.flex.viewer {
                        (filtered_paginator("player-pagination", "/api/v1/players/"))
                        p.viewer-welcome {
                            (tr("player-viewer.welcome"))
                        }
                        div.viewer-content {
                            div.flex.col{
                                h3 style = "font-size:1.1em; margin: 10px 0" {
                                    (tr("player-viewer"))
                                    i #player-player-id {}
                                    " - "
                                    i.fa.fa-pencil-alt.clickable #player-name-pen aria-hidden = "true" {} (PreEscaped("&nbsp;")) i #player-player-name {}
                                }
                                p {
                                    (tr("player-viewer.info"))
                                }
                                p.info-red.output style = "margin: 10px" {}
                                p.info-green.output style = "margin: 10px" {}
                                div.stats-container.flex.space {
                                    span {
                                        b {
                                            (tr("player-banned"))
                                        }
                                        br;
                                        div.dropdown-menu.js-search #edit-player-banned style = "max-width: 50px" {
                                            div {
                                                input type="text" style = "font-weight: bold;";
                                            }
                                            div.menu {
                                                ul {
                                                    li.white.hover data-value="true" {(tr("player-banned.yes"))}
                                                    li.white.hover data-value="false" {(tr("player-banned.no"))}
                                                }
                                            }
                                        }
                                    }
                                    span {
                                        b {
                                            (tr("player-nationality"))
                                        }
                                        br;
                                        p {
                                            (tr("player-nationality.info"))
                                        }
                                        div.dropdown-menu.js-search #edit-player-nationality data-default = "None" {
                                            div {
                                                input type="text" style = "font-weight: bold;";
                                            }
                                            div.menu {
                                                ul {
                                                    li.white.hover.underlined data-value = "None" {(tr("player-nationality.none")) }
                                                    @for nation in nationalities {
                                                        li.white.hover data-value = {(nation.iso_country_code)} data-display = {(nation.nation)} {
                                                            span class = "flag-icon" style={"background-image: url(/static/demonlist/images/flags/" (nation.iso_country_code.to_lowercase()) ".svg"} {}
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
                                div.stats-container.flex.space {
                                    span {
                                        b {
                                            (tr("player-subdivision"))
                                        }
                                        br;
                                        div.dropdown-menu.js-search #edit-player-subdivision data-default = "None" {
                                            div{
                                                input type="text" style = "color: #444446; font-weight: bold;";
                                            }
                                            div.menu {
                                                ul {
                                                    li.white.hover.underlined data-value = "None" {(tr("player-subdivision.none")) }
                                                }
                                            }
                                        }
                                    }
                                }
                                span.button.blue.hover #player-list-records style = "margin: 15px auto 0px" {(tr("player-viewer.records-redirect")) };
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
                (tr("player-idsearch-panel"))
            }
            p {
                (tr("player-idsearch-panel.info"))
            }
            form.flex.col #player-search-by-player-id-form novalidate = "" {
                p.info-red.output {}
                span.form-input #search-player-id {
                    label for = "id" {(tr("player-idsearch-panel.id-field")) }
                    input required = "" type = "number" name = "id" min = "0" style="width:93%";
                    p.error {}
                }
                input.button.blue.hover type = "submit" style = "margin: 15px auto 0px;" value=(tr("player-idsearch-panel.submit"));
            }
        }
    }
}

fn change_name_dialog() -> Markup {
    html! {
        div.overlay.closable {
            div.dialog #player-name-dialog {
                span.plus.cross.hover {}
                h2.underlined.pad {
                    (tr("player-name-dialog"))
                }
                p style = "max-width: 400px"{
                    (tr("player-name-dialog.info"))
                }
                form.flex.col novalidate = "" {
                    p.info-red.output {}
                    p.info-green.output {}
                    span.form-input #player-name-edit {
                        label for = "name" {(tr("player-name-dialog.name-field")) }
                        input name = "name" type = "text" required = "";
                        p.error {}
                    }
                    input.button.blue.hover type = "submit" style = "margin: 15px auto 0px;" value = (tr("player-name-dialog.submit"));
                }
            }
        }
    }
}
