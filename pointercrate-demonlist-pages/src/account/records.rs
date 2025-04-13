use crate::components::{
    demon_dropdown, player_selection_dialog,
    submitter::{submit_panel, RecordSubmitter},
};
use maud::{html, Markup, PreEscaped};
use pointercrate_core::{
    error::PointercrateError,
    localization::{ftr, tr},
    permission::PermissionsManager,
};
use pointercrate_core_pages::{
    error::ErrorFragment,
    util::{dropdown, paginator},
};
use pointercrate_demonlist::{
    demon::{current_list, Demon},
    LIST_HELPER,
};
use pointercrate_user::auth::{AuthenticatedUser, NonMutating};
use pointercrate_user_pages::account::AccountPageTab;
use sqlx::PgConnection;
use unic_langid::LanguageIdentifier;

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

    fn tab(&self, lang: &'static LanguageIdentifier) -> Markup {
        html! {
            b {
                (tr(lang, "records"))
            }
            (PreEscaped("&nbsp;&nbsp;"))
            i class = "fa fa-trophy fa-2x" aria-hidden="true" {}
        }
    }

    async fn content(
        &self, lang: &'static LanguageIdentifier, _user: &AuthenticatedUser<NonMutating>, _permissions: &PermissionsManager,
        connection: &mut PgConnection,
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
                (RecordSubmitter::new(false, &demons[..], lang))
                (record_manager(lang, &demons[..]))
                (note_adder(lang))
                div.panel.fade #record-notes-container style = "display:none" {
                    div.white.hover.clickable #add-record-note-open {
                        b {(tr(lang, "record-note"))}
                    }
                    div #record-notes {} // populated by javascript when a record is clicked
                }
                (manager_help(lang))
            }
            div.right {
                (status_selector(lang))
                (record_selector(lang))
                (player_selector(lang))
                (submit_panel(lang))
            }
            (change_progress_dialog(lang))
            (change_video_dialog(lang))
            (change_holder_dialog(lang))
            (change_demon_dialog(lang, &demons[..]))
        }
    }
}

fn record_manager(lang: &'static LanguageIdentifier, demons: &[Demon]) -> Markup {
    html! {
        div.panel.fade #record-manager {
            h2.underlined.pad {
                (tr(lang, "record-manager")) " - "
                (dropdown("All", html! {
                    li.white.hover.underlined data-value = "All"
                     {(tr(lang, "record-manager.all-option"))}
                }, demons.iter().map(|demon| html!(li.white.hover data-value = (demon.base.id) data-display = (demon.base.name) {b{"#"(demon.base.position) " - " (demon.base.name)} br; {"by "(demon.publisher.name)}}))))
            }
            div.flex.viewer {
                (paginator("record-pagination", "/api/v1/records/"))
                p.viewer-welcome {
                    (tr(lang, "record-viewer.welcome"))
                }
                div.viewer-content {
                    div.flex.col {
                        h3 style = "font-size:1.1em; margin-top: 10px" {
                            i.fa.fa-clipboard.clickable #record-copy-info aria-hidden = "true" {}
                            " " (tr(lang, "record-viewer"))
                            i #record-id {}
                            " - "
                            div.dropdown-menu.js-search #edit-record-status style = "max-width: 220px" {
                                div{
                                    input type="text" style = "color: #444446; font-weight: bold;";
                                }
                                div.menu {
                                    ul {
                                        li.white.hover data-value="approved" {(tr(lang, "record-approved"))}
                                        li.white.hover data-value="rejected" {(tr(lang, "record-rejected"))}
                                        li.white.hover data-value="under consideration" {(tr(lang, "record-underconsideration"))}
                                        li.white.hover data-value="submitted" {(tr(lang, "record-submitted"))}
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
                                    i.fa.fa-pencil-alt.clickable #record-video-pen aria-hidden = "true" {} " " (tr(lang, "record-videolink")) ":"
                                }
                                br;
                                a.link #record-video-link target = "_blank" {}
                            }
                        }
                        div.stats-container.flex.space {
                            span {
                                b { (tr(lang, "record-rawfootage")) ":" }
                                br;
                                a.link #record-raw-footage-link target = "_blank" {}
                            }
                        }
                        div.stats-container.flex.space {
                            span {
                                b {
                                    i.fa.fa-pencil-alt.clickable #record-demon-pen aria-hidden = "true" {} " " (tr(lang, "record-demon")) ":"
                                }
                                br;
                                span #record-demon {}
                            }
                            span {
                                b {
                                    i.fa.fa-pencil-alt.clickable #record-holder-pen aria-hidden = "true" {} " " (tr(lang, "record-holder")) ":"
                                }
                                br;
                                span #record-holder {}
                            }
                        }
                        div.stats-container.flex.space {
                            span {
                                b {
                                    i.fa.fa-pencil-alt.clickable #record-progress-pen aria-hidden = "true" {} " " (tr(lang, "record-progress")) ":"
                                }
                                br;
                                span #record-progress {}
                            }
                            span {
                                b {
                                    (tr(lang, "record-submitter")) ":"
                                }
                                br;
                                span #record-submitter {}
                            }
                        }
                        span.button.red.hover #record-delete style = "margin: 15px auto 0px" {(tr(lang, "record-viewer.delete"))};
                    }
                }

            }
        }
    }
}

fn manager_help(lang: &'static LanguageIdentifier) -> Markup {
    let states = vec![
        ("submitted", tr(lang, "record-submitted")),
        ("underConsideration", tr(lang, "record-underconsideration")),
        ("approved", tr(lang, "record-approved")),
        ("rejected", tr(lang, "record-rejected")),
    ];

    html! {
        div.panel.fade {
            h1.underlined.pad {
                (tr(lang, "record-manager-help"))
            }
            p {
                (tr(lang, "record-manager-help.a"))
            }
            p {
                (tr(lang, "record-manager-help.b"))
                ul {
                    li {
                        b {(tr(lang, "record-rejected")) ": "} (tr(lang, "record-manager-help.rejected"))
                    }
                    li {
                        b {(tr(lang, "record-approved")) ": "} (tr(lang, "record-manager-help.approved"))
                    }
                    li {
                        b {(tr(lang, "record-submitted")) ": "} (tr(lang, "record-manager-help.submitted"))
                    }
                    li {
                        b {(tr(lang, "record-underconsideration")) ": "} (tr(lang, "record-manager-help.underconsideration"))
                    }
                }
            }
            p {
                b { (tr(lang, "record-manager-help.note")) ": " }
                (tr(lang, "record-manager-help.note-a"))
            }
            p {
                b { (tr(lang, "record-manager-help.note")) ": " }
                (tr(lang, "record-manager-help.note-b"))
            }
        }
    }
}

fn status_selector(lang: &'static LanguageIdentifier) -> Markup {
    // FIXME: no vec
    let dropdown_items = vec![
        html! {
            li.white.hover data-value = "approved" {(tr(lang, "record-approved"))}
        },
        html! {
            li.white.hover data-value = "submitted" {(tr(lang, "record-submitted"))}
        },
        html! {
            li.white.hover data-value = "rejected" {(tr(lang, "record-rejected"))}
        },
        html! {
            li.white.hover data-value = "under consideration" {(tr(lang, "record-underconsideration"))}
        },
    ];

    html! {
        div.panel.fade #status-filter-panel style = "overflow: visible" {
            h2.underlined.pad {
                (tr(lang, "record-status-filter-panel"))
            }
            p {
                (tr(lang, "record-status-filter-panel.info"))
            }
            (dropdown("All", html! {
                li.white.hover.underlined data-value = "All" {(tr(lang, "record-status-filter-all"))}
            }, dropdown_items.into_iter()))
        }
    }
}

fn player_selector(lang: &'static LanguageIdentifier) -> Markup {
    html! {
        div.panel.fade {
            h2.underlined.pad {
                (tr(lang, "record-playersearch-panel"))
            }
            p {
                (tr(lang, "record-playersearch-panel.info"))
            }
            form.flex.col.underlined.pad #record-filter-by-player-id-form novalidate = "" {
                p.info-red.output {}
                span.form-input #record-player-id {
                    label for = "id" {(tr(lang, "record-playersearch-panel.id-field")) ":"}
                    input required = "" type = "number" name = "id" min = "0" style="width:93%"; // FIXME: I have no clue why the input thinks it's a special snowflake and fucks up its width, but I dont have the time to fix it
                    p.error {}
                }
                input.button.blue.hover type = "submit" style = "margin: 15px auto 0px;" value=(tr(lang, "record-playersearch-panel.id-submit"));
            }
            form.flex.col #record-filter-by-player-name-form novalidate = "" {
                p.info-red.output {}
                span.form-input #record-player-name {
                    label for = "name" {(tr(lang, "record-playersearch-panel.name-field")) ":"}
                    input required = "" type = "text" name = "name";
                    p.error {}
                }
                input.button.blue.hover type = "submit" style = "margin: 15px auto 0px;" value=(tr(lang, "record-playersearch-panel.name-submit"));
            }
        }
    }
}

fn record_selector(lang: &'static LanguageIdentifier) -> Markup {
    html! {
        div.panel.fade {
            h2.underlined.pad {
                (tr(lang, "record-idsearch-panel"))
            }
            p {
                (tr(lang, "record-idsearch-panel.info"))
            }
            form.flex.col #record-search-by-record-id-form novalidate = "" {
                p.info-red.output {}
                span.form-input #record-record-id {
                    label for = "id" {(tr(lang, "record-idsearch-panel.id-field")) ":"}
                    input required = "" type = "number" name = "id" min = "0" style="width:93%"; // FIXME: I have no clue why the input thinks it's a special snowflake and fucks up its width, but I dont have the time to fix it
                    p.error {}
                }
                input.button.blue.hover type = "submit" style = "margin: 15px auto 0px;" value=(tr(lang, "record-idsearch-panel.submit"));
            }
        }
    }
}

fn note_adder(lang: &'static LanguageIdentifier) -> Markup {
    html! {
        div.panel.fade.closable #add-record-note style = "display: none" {
            span.plus.cross.hover {}
            div style="display: flex;align-items: center;justify-content: space-between;" {
                div.button.blue.hover.small style = "width: 100px; margin-bottom: 10px"{
                    (tr(lang, "record-note.submit"))
                }
                div.cb-container.flex.no-stretch style="justify-content: space-between; align-items: center" {
                    b {
                        (tr(lang, "record-note.public-checkbox")) ":"
                    }
                    input #add-note-is-public-checkbox type = "checkbox" name = "is_public";
                    span.checkmark {}
                }
            }
            p.info-red.output {}
            textarea style = "width: 100%" placeholder = (tr(lang, "record-note.placeholder")) {}
        }
    }
}

fn change_progress_dialog(lang: &'static LanguageIdentifier) -> Markup {
    html! {
        div.overlay.closable {
            div.dialog #record-progress-dialog {
                span.plus.cross.hover {}
                h2.underlined.pad {
                    (tr(lang, "record-progress-dialog")) ":"
                }
                p style = "max-width: 400px"{
                    (tr(lang, "record-progress-dialog.info"))
                }
                form.flex.col novalidate = "" {
                    p.info-red.output {}
                    p.info-green.output {}
                    span.form-input #record-progress-edit {
                        label for = "progress" {(tr(lang, "record-progress-dialog.progress-field")) ":"}
                        input name = "progress" type = "number" min = "0" max="100" required = "";
                        p.error {}
                    }
                    input.button.blue.hover type = "submit" style = "margin: 15px auto 0px;" value = (tr(lang, "record-progress-dialog.submit"));
                }
            }
        }
    }
}

fn change_video_dialog(lang: &'static LanguageIdentifier) -> Markup {
    html! {
        div.overlay.closable {
            div.dialog #record-video-dialog {
                span.plus.cross.hover {}
                h2.underlined.pad {
                    (tr(lang, "record-videolink-dialog")) ":"
                }
                p style = "max-width: 400px"{
                    (tr(lang, "record-videolink-dialog.info"))
                }
                form.flex.col novalidate = "" {
                    p.info-red.output {}
                    p.info-green.output {}
                    span.form-input #record-video-edit {
                        label for = "video" {(tr(lang, "record-videolink-dialog.videolink-field")) ":"}
                        input name = "video" type = "url";
                        p.error {}
                    }
                    input.button.blue.hover type = "submit" style = "margin: 15px auto 0px;" value = (tr(lang, "record-videolink-dialog.submit"));
                }
            }
        }
    }
}

fn change_holder_dialog(lang: &'static LanguageIdentifier) -> Markup {
    player_selection_dialog(
        "record-holder-dialog",
        "_edit-holder-record",
        &(tr(lang, "record-holder-dialog") + ":"),
        &tr(lang, "record-holder-dialog.info"),
        &(tr(lang, "record-holder-dialog.submit")),
        "player",
    )
}

fn change_demon_dialog(lang: &'static LanguageIdentifier, demons: &[Demon]) -> Markup {
    html! {
        div.overlay.closable {
            div.dialog #record-demon-dialog style="overflow: initial;" {
                span.plus.cross.hover {}
                h2.underlined.pad {
                    (tr(lang, "record-demon-dialog")) ":"
                }
                div.flex.col {
                    p {
                        (tr(lang, "record-videolink-dialog.info"))
                    }
                    (demon_dropdown("edit-demon-record", demons.iter()))
                }
            }
        }
    }
}
