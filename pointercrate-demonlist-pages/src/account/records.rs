use crate::{
    components::{demon_dropdown, player_selection_dialog},
    submit_record::submit_record_panel,
};
use maud::{html, Markup, PreEscaped};
use pointercrate_core::{error::PointercrateError, localization::tr, permission::PermissionsManager, trp};
use pointercrate_core_pages::{
    error::ErrorFragment,
    util::{dropdown, paginator},
};
use pointercrate_demonlist::{
    demon::{current_list, Demon},
    player::DatabasePlayer,
    LIST_HELPER,
};
use pointercrate_user::auth::{AuthenticatedUser, NonMutating};
use pointercrate_user_pages::account::AccountPageTab;
use sqlx::PgConnection;

pub struct RecordsPage;

#[async_trait::async_trait]
impl AccountPageTab for RecordsPage {
    fn should_display_for(&self, permissions_we_have: u16, permissions: &PermissionsManager) -> bool {
        permissions.require_permission(permissions_we_have, LIST_HELPER).is_ok()
    }

    fn initialization_script(&self) -> String {
        "/static/demonlist/js/account/records.js".into()
    }

    fn tab_id(&self) -> u8 {
        3
    }

    fn tab(&self) -> Markup {
        html! {
            b {
                (tr("records"))
            }
            (PreEscaped("&nbsp;&nbsp;"))
            i class = "fa fa-trophy fa-2x" aria-hidden="true" {}
        }
    }

    async fn content(
        &self, _user: &AuthenticatedUser<NonMutating>, _permissions: &PermissionsManager, connection: &mut PgConnection,
    ) -> Markup {
        let demons = match current_list(connection).await {
            Ok(demons) => demons,
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
                (record_manager(&demons[..]))
                (note_adder())
                div.panel.fade #record-notes-container style = "display:none" {
                    div.white.hover.clickable #add-record-note-open {
                        b {(tr("record-note"))}
                    }
                    div #record-notes {} // populated by javascript when a record is clicked
                }
                (manager_help())
            }
            div.right {
                (status_selector())
                (record_selector())
                (player_selector())
                (submit_record_panel(None))
            }
            (change_progress_dialog())
            (change_video_dialog())
            (change_holder_dialog())
            (change_demon_dialog(&demons[..]))
        }
    }
}

fn record_manager(demons: &[Demon]) -> Markup {
    html! {
        div.panel.fade #record-manager {
            h2.underlined.pad {
                (tr("record-manager")) " - "
                (dropdown("All", html! {
                    li.white.hover.underlined data-value = "All"
                     {(tr("record-manager.all-option"))}
                }, demons.iter().map(|demon| html!(li.white.hover data-value = (demon.base.id) data-display = (demon.base.name) {b{"#"(demon.base.position) " - " (demon.base.name)} br; {(trp!("demon-listed.publisher", "publisher" = demon.publisher.name))}}))))
            }
            div.flex.viewer {
                (paginator("record-pagination", "/api/v1/records/"))
                p.viewer-welcome {
                    (tr("record-viewer.welcome"))
                }
                div.viewer-content {
                    div.flex.col {
                        h3 style = "font-size:1.1em; margin-top: 10px" {
                            i.fa.fa-clipboard.clickable #record-copy-info aria-hidden = "true" {}
                            " " (tr("record-viewer"))
                            i #record-id {}
                            " - "
                            div.dropdown-menu.js-search #edit-record-status style = "max-width: 220px" {
                                div{
                                    input type="text" style = "font-weight: bold;";
                                }
                                div.menu {
                                    ul {
                                        li.white.hover data-value="approved" {(tr("record-approved"))}
                                        li.white.hover data-value="rejected" {(tr("record-rejected"))}
                                        li.white.hover data-value="under consideration" {(tr("record-underconsideration"))}
                                        li.white.hover data-value="submitted" {(tr("record-submitted"))}
                                    }
                                }
                            }
                        }

                        iframe."ratio-16-9"#record-video style="width:90%; margin: 15px 5%" allowfullscreen="" {"Video"}
                        p.info-red.output style = "margin: 10px" {}
                        p.info-green.output style = "margin: 10px" {}
                        div.stats-container.flex.space  {
                            span {
                                b {
                                    i.fa.fa-pencil-alt.clickable #record-video-pen aria-hidden = "true" {} " " (tr("record-videolink"))
                                }
                                br;
                                a.link #record-video-link target = "_blank" {}
                            }
                        }
                        div.stats-container.flex.space {
                            span {
                                b { (tr("record-rawfootage"))  }
                                br;
                                a.link #record-raw-footage-link target = "_blank" {}
                            }
                        }
                        div.stats-container.flex.space {
                            span {
                                b {
                                    i.fa.fa-pencil-alt.clickable #record-demon-pen aria-hidden = "true" {} " " (tr("record-demon"))
                                }
                                br;
                                span #record-demon {}
                            }
                            span {
                                b {
                                    i.fa.fa-pencil-alt.clickable #record-holder-pen aria-hidden = "true" {} " " (tr("record-holder"))
                                }
                                br;
                                span #record-holder {}
                            }
                        }
                        div.stats-container.flex.space {
                            span {
                                b {
                                    i.fa.fa-pencil-alt.clickable #record-progress-pen aria-hidden = "true" {} " " (tr("record-progress"))
                                }
                                br;
                                span #record-progress {}
                            }
                            span {
                                b {
                                    (tr("record-submitter"))
                                }
                                br;
                                span #record-submitter {}
                            }
                        }
                        span.button.red.hover #record-delete style = "margin: 15px auto 0px" {(tr("record-viewer.delete"))};
                    }
                }

            }
        }
    }
}

fn manager_help() -> Markup {
    html! {
        div.panel.fade {
            h1.underlined.pad {
                (tr("record-manager-help"))
            }
            p {
                (tr("record-manager-help.a"))
            }
            p {
                (PreEscaped(tr("record-manager-help.b")))
                ul {
                    li {
                        b {(tr("record-rejected")) ": "} (PreEscaped(tr("record-manager-help.rejected")))
                    }
                    li {
                        b {(tr("record-approved")) ": "} (PreEscaped(tr("record-manager-help.approved")))
                    }
                    li {
                        b {(tr("record-submitted")) ": "} (PreEscaped(tr("record-manager-help.submitted")))
                    }
                    li {
                        b {(tr("record-underconsideration")) ": "} (PreEscaped(tr("record-manager-help.underconsideration")))
                    }
                }
            }
            p {
                b { (tr("record-manager-help.note")) ": " }
                (PreEscaped(tr("record-manager-help.note-a")))
            }
            p {
                b { (tr("record-manager-help.note")) ": " }
                (PreEscaped(tr("record-manager-help.note-b")))
            }
        }
    }
}

fn status_selector() -> Markup {
    // FIXME: no vec
    let dropdown_items = vec![
        html! {
            li.white.hover data-value = "approved" {(tr("record-approved"))}
        },
        html! {
            li.white.hover data-value = "submitted" {(tr("record-submitted"))}
        },
        html! {
            li.white.hover data-value = "rejected" {(tr("record-rejected"))}
        },
        html! {
            li.white.hover data-value = "under consideration" {(tr("record-underconsideration"))}
        },
    ];

    html! {
        div.panel.fade #status-filter-panel style = "overflow: visible" {
            h2.underlined.pad {
                (tr("record-status-filter-panel"))
            }
            p {
                (tr("record-status-filter-panel.info"))
            }
            (dropdown("All", html! {
                li.white.hover.underlined data-value = "All" {(tr("record-status-filter-all"))}
            }, dropdown_items.into_iter()))
        }
    }
}

fn player_selector() -> Markup {
    html! {
        div.panel.fade {
            h2.underlined.pad {
                (tr("record-playersearch-panel"))
            }
            p {
                (tr("record-playersearch-panel.info"))
            }
            form.flex.col.underlined.pad #record-filter-by-player-id-form novalidate = "" {
                p.info-red.output {}
                span.form-input #record-player-id {
                    label for = "id" {(tr("record-playersearch-panel.id-field")) }
                    input required = "" type = "number" name = "id" min = "0" style="width:93%"; // FIXME: I have no clue why the input thinks it's a special snowflake and fucks up its width, but I dont have the time to fix it
                    p.error {}
                }
                input.button.blue.hover type = "submit" style = "margin: 15px auto 0px;" value=(tr("record-playersearch-panel.id-submit"));
            }
            form.flex.col #record-filter-by-player-name-form novalidate = "" {
                p.info-red.output {}
                span.form-input #record-player-name {
                    label for = "name" {(tr("record-playersearch-panel.name-field")) }
                    input required = "" type = "text" name = "name";
                    p.error {}
                }
                input.button.blue.hover type = "submit" style = "margin: 15px auto 0px;" value=(tr("record-playersearch-panel.name-submit"));
            }
        }
    }
}

fn record_selector() -> Markup {
    html! {
        div.panel.fade {
            h2.underlined.pad {
                (tr("record-idsearch-panel"))
            }
            p {
                (tr("record-idsearch-panel.info"))
            }
            form.flex.col #record-search-by-record-id-form novalidate = "" {
                p.info-red.output {}
                span.form-input #record-record-id {
                    label for = "id" {(tr("record-idsearch-panel.id-field")) }
                    input required = "" type = "number" name = "id" min = "0" style="width:93%"; // FIXME: I have no clue why the input thinks it's a special snowflake and fucks up its width, but I dont have the time to fix it
                    p.error {}
                }
                input.button.blue.hover type = "submit" style = "margin: 15px auto 0px;" value=(tr("record-idsearch-panel.submit"));
            }
        }
    }
}

fn note_adder() -> Markup {
    html! {
        div.panel.fade.closable #add-record-note style = "display: none" {
            span.plus.cross.hover {}
            div style="display: flex;align-items: center;justify-content: space-between;" {
                div.button.blue.hover.small style = "width: 100px; margin-bottom: 10px"{
                    (tr("record-note.submit"))
                }
                div.cb-container.flex.no-stretch style="justify-content: space-between; align-items: center" {
                    b {
                        (tr("record-note.public-checkbox"))
                    }
                    input #add-note-is-public-checkbox type = "checkbox" name = "is_public";
                    span.checkmark {}
                }
            }
            p.info-red.output {}
            textarea style = "width: 100%" placeholder = (tr("record-note.placeholder")) {}
        }
    }
}

fn change_progress_dialog() -> Markup {
    html! {
        div.overlay.closable {
            div.dialog #record-progress-dialog {
                span.plus.cross.hover {}
                h2.underlined.pad {
                    (tr("record-progress-dialog"))
                }
                p style = "max-width: 400px"{
                    (tr("record-progress-dialog.info"))
                }
                form.flex.col novalidate = "" {
                    p.info-red.output {}
                    p.info-green.output {}
                    span.form-input #record-progress-edit {
                        label for = "progress" {(tr("record-progress-dialog.progress-field")) }
                        input name = "progress" type = "number" min = "0" max="100" required = "";
                        p.error {}
                    }
                    input.button.blue.hover type = "submit" style = "margin: 15px auto 0px;" value = (tr("record-progress-dialog.submit"));
                }
            }
        }
    }
}

fn change_video_dialog() -> Markup {
    html! {
        div.overlay.closable {
            div.dialog #record-video-dialog {
                span.plus.cross.hover {}
                h2.underlined.pad {
                    (tr("record-videolink-dialog"))
                }
                p style = "max-width: 400px"{
                    (tr("record-videolink-dialog.info"))
                }
                form.flex.col novalidate = "" {
                    p.info-red.output {}
                    p.info-green.output {}
                    span.form-input #record-video-edit {
                        label for = "video" {(tr("record-videolink-dialog.videolink-field")) }
                        input name = "video" type = "url";
                        p.error {}
                    }
                    input.button.blue.hover type = "submit" style = "margin: 15px auto 0px;" value = (tr("record-videolink-dialog.submit"));
                }
            }
        }
    }
}

fn change_holder_dialog() -> Markup {
    player_selection_dialog(
        "record-holder-dialog",
        "_edit-holder-record",
        &tr("record-holder-dialog"),
        &tr("record-holder-dialog.info"),
        &tr("record-holder-dialog.submit"),
        "player",
        &None,
    )
}

fn change_demon_dialog(demons: &[Demon]) -> Markup {
    html! {
        div.overlay.closable {
            div.dialog #record-demon-dialog style="overflow: initial;" {
                span.plus.cross.hover {}
                h2.underlined.pad {
                    (tr("record-demon-dialog"))
                }
                div.flex.col {
                    p {
                        (tr("record-videolink-dialog.info"))
                    }
                    (demon_dropdown("edit-demon-record", demons, None))
                }
            }
        }
    }
}
