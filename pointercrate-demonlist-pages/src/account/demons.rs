use crate::components::{player_selection_dialog, player_selection_dropdown};
use maud::{html, Markup, PreEscaped};
use pointercrate_core::{localization::tr, permission::PermissionsManager};
use pointercrate_core_pages::trp_html;
use pointercrate_core_pages::util::filtered_paginator;
use pointercrate_demonlist::LIST_MODERATOR;
use pointercrate_user::auth::{AuthenticatedUser, NonMutating};
use pointercrate_user_pages::account::AccountPageTab;
use sqlx::PgConnection;

pub struct DemonsTab;

#[async_trait::async_trait]
impl AccountPageTab for DemonsTab {
    fn should_display_for(&self, permissions_we_have: u16, permissions: &PermissionsManager) -> bool {
        permissions.require_permission(permissions_we_have, LIST_MODERATOR).is_ok()
    }

    fn initialization_script(&self) -> String {
        "/static/demonlist/js/account/demon.js".into()
    }

    fn tab_id(&self) -> u8 {
        5
    }

    fn tab(&self) -> Markup {
        html! {
            i class = "fa fa-shower fa-2x" aria-hidden="true" {}
            (PreEscaped("&nbsp;&nbsp;"))
            b {
                (tr("demons"))
            }
        }
    }

    async fn content(
        &self, _user: &AuthenticatedUser<NonMutating>, _permissions: &PermissionsManager, _connection: &mut PgConnection,
    ) -> Markup {
        html! {
            div.left {
                (demon_submitter())
                div.panel.fade {
                    h2.underlined.pad {
                        (tr("demon-manager"))
                    }
                    div.flex.viewer {
                        (filtered_paginator("demon-pagination", "/api/v2/demons/listed/"))
                        p.viewer-welcome {
                            (tr("demon-viewer.welcome"))
                        }

                        div.viewer-content {
                            div.flex.col{
                                h3 style = "font-size:1.1em; margin: 10px 0" {
                                    (tr("demon-viewer"))
                                    i #demon-demon-id {}
                                    " - "
                                    i.fa.fa-pencil-alt.clickable #demon-name-pen aria-hidden = "true" {} (PreEscaped("&nbsp;")) i #demon-demon-name {}
                                }

                                iframe."ratio-16-9"#demon-video style="width:90%; margin: 15px 5%" allowfullscreen="" {(tr("demon-video"))}
                                p.info-red.output style = "margin: 10px" {}
                                p.info-green.output style = "margin: 10px" {}
                                div.stats-container.flex.space  {
                                    span{
                                        b {
                                            i.fa.fa-pencil-alt.clickable #demon-video-pen aria-hidden = "true" {} " " (tr("demon-viewer.video-field"))
                                        }
                                        br;
                                        a.link #demon-video-link target = "_blank" {}
                                    }
                                }
                                div.stats-container.flex.space  {
                                    span{
                                        b {
                                            i.fa.fa-pencil-alt.clickable #demon-thumbnail-pen aria-hidden = "true" {} " " (tr("demon-viewer.thumbnail-field"))
                                        }
                                        br;
                                        a.link #demon-thumbnail-link target = "_blank" {}
                                    }
                                }
                                div.stats-container.flex.space  {
                                    span{
                                        b {
                                            i.fa.fa-pencil-alt.clickable #demon-position-pen aria-hidden = "true" {} " " (tr("demon-viewer.position-field"))
                                        }
                                        br;
                                        span #demon-position {}
                                    }
                                    span{
                                        b {
                                            i.fa.fa-pencil-alt.clickable #demon-requirement-pen aria-hidden = "true" {} " " (tr("demon-viewer.requirement-field"))
                                        }
                                        br;
                                        span #demon-requirement {}
                                    }
                                }
                                div.stats-container.flex.space  {
                                    span{
                                        b {
                                            i.fa.fa-pencil-alt.clickable #demon-publisher-pen aria-hidden = "true" {} " " (tr("demon-viewer.publisher-field"))
                                        }
                                        br;
                                        span #demon-publisher {}
                                    }
                                    span{
                                        b {
                                            i.fa.fa-pencil-alt.clickable #demon-verifier-pen aria-hidden = "true" {} " " (tr("demon-viewer.verifier-field"))
                                        }
                                        br;
                                        span #demon-verifier {}
                                    }
                                }
                                div.stats-container.flex.space  {
                                    span{
                                        i.fa.fa-plus.clickable #demon-add-creator-pen aria-hidden = "true" {} b {
                                            " " (tr("demon-viewer.creators-field"))
                                        }
                                        br;
                                        span #demon-creators {}

                                    }
                                }
                            }
                        }
                    }
                }
                div style="height: 50px" {} // to make sure that the footer doesnt float. if it floats, the user page is the only one without a scrollbar at the right, which causes jumpiness when switching tabs.
            }
            div.right {
                (submit_panel())
            }
            (change_name_dialog())
            (change_position_dialog())
            (change_requirement_dialog())
            (change_video_dialog())
            (change_thumbnail_dialog())
            (change_verifier_dialog())
            (change_publisher_dialog())
            (add_creator_dialog())
        }
    }
}

pub(super) fn submit_panel() -> Markup {
    html! {
        section.panel.fade.js-scroll-anim data-anim = "fade" {
            div.underlined {
                h2 {
                    (tr("demon-add-panel"))
                }
            }
            a.blue.hover.button.js-scroll data-destination = "demon-submitter" data-reveal = "true" {
                (tr("demon-add-panel.button"))
            }
        }
    }
}

fn change_name_dialog() -> Markup {
    html! {
        div.overlay.closable {
            div.dialog #demon-name-dialog {
                span.plus.cross.hover {}
                h2.underlined.pad {
                    (tr("demon-name-dialog"))
                }
                p style = "max-width: 400px"{
                    (tr("demon-name-dialog.info"))
                }
                form.flex.col novalidate = "" {
                    p.info-red.output {}
                    p.info-green.output {}
                    span.form-input #demon-name-edit {
                        label for = "name" {(tr("demon-name-dialog.name-field")) }
                        input name = "name" type = "text" required = "";
                        p.error {}
                    }
                    input.button.blue.hover type = "submit" style = "margin: 15px auto 0px;" value = (tr("demon-name-dialog.submit"));
                }
            }
        }
    }
}

fn change_requirement_dialog() -> Markup {
    html! {
        div.overlay.closable {
            div.dialog #demon-requirement-dialog {
                span.plus.cross.hover {}
                h2.underlined.pad {
                    (tr("demon-requirement-dialog"))
                }
                p style = "max-width: 400px"{
                    (tr("demon-requirement-dialog.info"))
                }
                form.flex.col novalidate = "" {
                    p.info-red.output {}
                    p.info-green.output {}
                    span.form-input #demon-requirement-edit {
                        label for = "requirement" {(tr("demon-requirement-dialog.requirement-field")) }
                        input name = "requirement" type = "number" min = "0" max="100" required = "";
                        p.error {}
                    }
                    input.button.blue.hover type = "submit" style = "margin: 15px auto 0px;" value = (tr("demon-requirement-dialog.submit"));
                }
            }
        }
    }
}

fn change_position_dialog() -> Markup {
    html! {
        div.overlay.closable {
            div.dialog #demon-position-dialog {
                span.plus.cross.hover {}
                h2.underlined.pad {
                    (tr("demon-position-dialog"))
                }
                p style = "max-width: 400px"{
                    (tr("demon-position-dialog.info"))
                }
                form.flex.col novalidate = "" {
                    p.info-red.output {}
                    p.info-green.output {}
                    span.form-input #demon-position-edit {
                        label for = "position" {(tr("demon-position-dialog.position-field")) }
                        input name = "position" type = "number" min = "1" required = "";
                        p.error {}
                    }
                    input.button.blue.hover type = "submit" style = "margin: 15px auto 0px;" value = (tr("demon-position-dialog.submit"));
                }
            }
        }
    }
}

fn change_video_dialog() -> Markup {
    html! {
        div.overlay.closable {
            div.dialog #demon-video-dialog {
                span.plus.cross.hover {}
                h2.underlined.pad {
                    (tr("demon-video-dialog"))
                }
                p style = "max-width: 400px"{
                    (tr("demon-video-dialog.info"))
                }
                form.flex.col novalidate = "" {
                    p.info-red.output {}
                    p.info-green.output {}
                    span.form-input #demon-video-edit {
                        label for = "video" {(tr("demon-video-dialog.video-field")) }
                        input name = "video" type = "url";
                        p.error {}
                    }
                    input.button.blue.hover type = "submit" style = "margin: 15px auto 0px;" value = (tr("demon-video-dialog.submit"));
                }
            }
        }
    }
}

fn change_thumbnail_dialog() -> Markup {
    html! {
        div.overlay.closable {
            div.dialog #demon-thumbnail-dialog {
                span.plus.cross.hover {}
                h2.underlined.pad {
                    (tr("demon-thumbnail-dialog"))
                }
                p style = "max-width: 400px"{
                    (trp_html!(
                        "demon-thumbnail-dialog.info",
                        "video-id" = html! {
                            "https://i.ytimg.com/vi/"
                            i { (tr("demon-thumbnail-dialog.info-videoid")) }
                            "/mqdefault.jpg"
                        }
                    ))
                }
                form.flex.col novalidate = "" {
                    p.info-red.output {}
                    p.info-green.output {}
                    span.form-input #demon-thumbnail-edit {
                        label for = "thumbnail" {(tr("demon-thumbnail-dialog.thumbnail-field")) }
                        input required="" name = "thumbnail" type = "url";
                        p.error {}
                    }
                    input.button.blue.hover type = "submit" style = "margin: 15px auto 0px;" value = (tr("demon-thumbnail-dialog.submit"));
                }
            }
        }
    }
}

fn change_verifier_dialog() -> Markup {
    player_selection_dialog(
        "demon-verifier-dialog",
        "demon-verifier-edit",
        &tr("demon-verifier-dialog"),
        &tr("demon-verifier-dialog.info"),
        &tr("demon-verifier-dialog.submit"),
        "verifier",
        &None,
    )
}

fn change_publisher_dialog() -> Markup {
    player_selection_dialog(
        "demon-publisher-dialog",
        "demon-publisher-edit",
        &tr("demon-publisher-dialog"),
        &tr("demon-publisher-dialog.info"),
        &tr("demon-publisher-dialog.submit"),
        "publisher",
        &None,
    )
}

fn add_creator_dialog() -> Markup {
    player_selection_dialog(
        "demon-add-creator-dialog",
        "demon-creator-add",
        &tr("demon-creator-dialog"),
        &tr("demon-creator-dialog.info"),
        &tr("demon-creator-dialog.submit"),
        "creator",
        &None,
    )
}

fn demon_submitter() -> Markup {
    html! {
        section.panel.fade.closable #demon-submitter style = "display: none" {
            span.plus.cross.hover {}
            div.flex {
                form #demon-submission-form novalidate = "" {
                    div.underlined {
                        h2 {(tr("demon-add-form")) }
                    }
                    p.info-red.output {}
                    p.info-green.output {}
                    span.form-input.flex.col #demon-add-name {
                        label for = "name" {
                            (tr("demon-add-form.name-field"))
                        }
                        input type = "text" name = "name" required="";
                        p.error {}
                    }
                    span.form-input.flex.col #demon-add-level-id {
                        label for = "level_id" {
                            (tr("demon-add-form.levelid-field"))
                        }
                        input type = "number" name = "level_id" min = "1";
                        p.error {}
                    }
                    span.form-input.flex.col #demon-add-position {
                        label for = "position" {
                            (tr("demon-add-form.position-field"))
                        }
                        input type = "number" name = "position" required="" min="1";
                        p.error {}
                    }
                    span.form-input.flex.col #demon-add-requirement {
                        label for = "requirement" {
                            (tr("demon-add-form.requirement-field"))
                        }
                        input type = "number" name = "requirement" required="" min="0" max = "100";
                        p.error {}
                    }
                    span.form-input.flex.col data-type = "dropdown" {
                        label{(tr("demon-add-form.verifier-field")) }
                        br;
                        (player_selection_dropdown("demon-add-verifier", "/api/v1/players/", "name", "verifier", &None))
                        p.error {}
                    }
                    span.form-input.flex.col data-type = "dropdown" {
                        label {(tr("demon-add-form.publisher-field")) }
                        br;
                        (player_selection_dropdown("demon-add-publisher", "/api/v1/players/", "name", "publisher", &None))
                        p.error {}
                    }
                    span.form-input.flex.col #demon-add-video {
                        label for = "video" {
                            (tr("demon-add-form.video-field"))
                        }
                        input type = "url" name = "video";
                        p.error {}
                    }
                    span {
                        i.fa.fa-plus.clickable #add-demon-add-creator-pen aria-hidden = "true" {} i {
                            " " (tr("demon-add-form.creators-field"))
                        }
                        span #demon-add-creators {}
                    }
                    input.button.blue.hover type = "submit" style = "margin: 15px auto 0px;" value=(tr("demon-add-form.submit")) ;
                }
            }
        }
    }
}
