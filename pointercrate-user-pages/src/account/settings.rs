use crate::account::AccountPageTab;
use maud::{html, Markup, PreEscaped};
use pointercrate_core::{
    localization::{task_lang, tr},
    permission::PermissionsManager,
    trp,
};
use pointercrate_core_pages::head::Script;
use pointercrate_user::{
    auth::{AuthenticatedUser, NonMutating},
    config,
};
use sqlx::PgConnection;

pub struct SettingsTab;

#[async_trait::async_trait]
impl AccountPageTab for SettingsTab {
    fn should_display_for(&self, _permissions_we_have: u16, _permissions: &PermissionsManager) -> bool {
        true
    }

    fn initialization_script(&self) -> String {
        "/static/user/js/account/settings.js".into()
    }

    fn additional_scripts(&self) -> Vec<Script> {
        if cfg!(feature = "oauth2") {
            vec![Script::r#async("https://accounts.google.com/gsi/client")]
        } else {
            Vec::new()
        }
    }

    fn tab_id(&self) -> u8 {
        1
    }

    fn tab(&self) -> Markup {
        html! {
            b {
                (tr("settings"))
            }
            (PreEscaped("&nbsp;&nbsp;"))
            i class = "fa fa-cog fa-2x" aria-hidden="true" {}
        }
    }

    async fn content(
        &self, authenticated_user: &AuthenticatedUser<NonMutating>, permissions: &PermissionsManager, _connection: &mut PgConnection,
    ) -> Markup {
        let user = authenticated_user.user();

        let is_elevated = permissions.is_elevated(user.permissions);

        let permissions = permissions.bits_to_permissions(user.permissions);
        let permission_string = permissions.iter().map(|perm| tr(perm.text_id())).collect::<Vec<_>>().join(", ");

        let lang = task_lang();

        html! {
            div.left {
                div.panel.fade {
                    h1.underlined.pad {
                        (trp!("settings.header", "username" = user.name()))
                    }
                    div.flex.space.wrap #things {
                        p.info-red.output style = "margin: 10px" {}
                        p.info-green.output style = "margin: 10px" {}
                        span {
                            b {
                                (tr("settings-username")) ": "
                            }
                            (user.name)
                            p {
                                (tr("settings-username.info"))
                            }
                        }
                        @if is_elevated {
                            span {
                                b {
                                    i.fa.fa-pencil-alt.clickable #display-name-pen aria-hidden = "true" {} " " (tr("settings-display-name")) ": "
                                }
                                i #profile-display-name {
                                    @match user.display_name {
                                        Some(ref dn) => (dn),
                                        None => "-"
                                    }
                                }
                                p {
                                    (tr("settings-display-name.info"))
                                }
                            }
                            span {
                                b {
                                    i.fa.fa-pencil-alt.clickable #youtube-pen aria-hidden = "true" {} " " (tr("settings-youtube")) ": "
                                }
                                i #profile-youtube-channel {
                                    @match user.youtube_channel {
                                        Some(ref yc) => a.link href = (yc) {},
                                        None => "-"
                                    }
                                }
                                p {
                                    (tr("settings-youtube.info"))
                                }
                            }
                            span {
                                b {
                                    (tr("settings-permissions")) ": "
                                }
                                (permission_string)
                                p {
                                    (tr("settings-permissions.info"))
                                }
                            }
                        }
                    }
                    div.flex.no-stretch {
                        input.button.red.hover #delete-account type = "button" style = "margin: 15px auto 0px;" value=(tr("settings-delete-account"));
                        @if authenticated_user.is_legacy() {
                            input.button.blue.hover #change-password type = "button" style = "margin: 15px auto 0px;" value=(tr("settings-change-password"));
                        }
                    }
                }
            }
            div.right {
                div.panel.fade {
                    h2.underlined.pad {
                        (tr("settings-logout"))
                    }
                    p {
                        (tr("settings-logout.info"))
                    }
                    a.red.hover.button href = "/logout" style = "margin: 15px auto 0px; display: inline-block" {
                        (tr("settings-logout.button"))
                    }
                }
                @if cfg!(feature = "oauth2") && authenticated_user.is_legacy() {
                    div.panel.fade {
                        h2.underlined.pad {
                            (tr("settings-oauth"))
                        }
                        p {
                            (tr("settings-oauth.info"))
                        }
                        div #g_id_onload
                            data-ux_mode="popup"
                            data-auto_select="true"
                            data-itp_support="true"
                            data-client_id=(config::google_client_id())
                            data-callback="googleOauthCallback" {}

                        script src=(format!("https://accounts.google.com/gsi/client?hl={}", &lang)) async {}
                        div .g_id_signin data-text="continue_with" style="margin: 10px 0px" data-locale=(lang) {}
                        p.error #g-signin-error style="text-align: left" {}
                    }
                }
                div.panel.fade {
                    h2.underlined.pad {
                        (tr("settings-get-token"))
                    }
                    p {
                        (tr("settings-get-token.info"))
                    }
                    div.overlined.pad #token-area style = "display: none" {
                        b {(tr("settings-get-token.view-header")) }
                        textarea #access-token readonly="" style = "resize: none; width: 100%; margin-top: 8px; min-height:75px" {}
                    }
                    form.flex.col #get-token-form novalidate = "" {
                        p.info-red.output {}
                        input.blue.hover.button type = "submit" style = "margin: 15px auto 0px;" value=(tr("settings-get-token.button"));
                    }
                }
                div.panel.fade {
                    h2.underlined.pad {
                        (tr("settings-invalidate-tokens"))
                    }
                    p {
                        (tr("settings-invalidate-tokens.info"))
                    }
                    form.flex.col #invalidate-form novalidate = "" {
                        p.info-red.output {}
                        input.blue.hover.button type = "submit" style = "margin: 15px auto 0px;" value=(tr("settings-invalidate-tokens.button"));
                    }
                }
            }
            (edit_display_name_dialog())
            (edit_youtube_link_dialog())
            @if authenticated_user.is_legacy() {
                (change_password_dialog())
            }
            (delete_account_dialog())
        }
    }
}

fn edit_display_name_dialog() -> Markup {
    html! {
        div.overlay.closable {
            div.dialog #edit-dn-dialog {
                span.plus.cross.hover {}
                h2.underlined.pad {
                    (tr("settings-display-name.dialog-header"))
                }
                form.flex.col novalidate = "" {
                    p.info-red.output {}
                    p.info-green.output {}
                    span.form-input #edit-dn {
                        label for = "display_name" {(tr("settings-display-name.dialog-newname")) }
                        input type = "text" name = "display_name";
                        p.error {}
                    }
                    input.button.blue.hover type = "submit" style = "margin: 15px auto 0px;" value=(tr("settings-display-name.dialog-submit"));
                }
            }
        }
    }
}

fn edit_youtube_link_dialog() -> Markup {
    html! {
        div.overlay.closable {
            div.dialog #edit-yt-dialog {
                span.plus.cross.hover {}
                h2.underlined.pad {
                    (tr("settings-youtube.dialog-header"))
                }
                form.flex.col novalidate = "" {
                    p.info-red.output {}
                    p.info-green.output {}
                    span.form-input #edit-yt {
                        label for = "youtube_channel" {(tr("settings-youtube.dialog-newlink")) }
                        input type = "url" name = "youtube_channel";
                        p.error {}
                    }
                    input.button.blue.hover type = "submit" style = "margin: 15px auto 0px;" value=(tr("settings-youtube.dialog-submit"));
                }
            }
        }
    }
}

fn change_password_dialog() -> Markup {
    html! {
        div.overlay.closable {
            div.dialog #edit-pw-dialog {
                span.plus.cross.hover {}
                h2.underlined.pad {
                    (tr("settings-change-password.dialog-header"))
                }
                p {
                    (tr("settings-change-password.dialog-info"))
                }
                form.flex.col novalidate = "" {
                    p.info-red.output {}
                    p.info-green.output {}
                    span.form-input #edit-pw {
                        label for = "password" {(tr("settings-change-password.dialog-newpassword")) }
                        input type = "password" name = "password" minlength = "10";
                        p.error {}
                    }
                    span.form-input #edit-pw-repeat {
                        label for = "password2" {(tr("settings-change-password.dialog-repeatnewpassword")) }
                        input type = "password"  minlength = "10";
                        p.error {}
                    }
                    span.overlined.pad.form-input #auth-pw {
                        label {(tr("settings-change-password.dialog-authenticate")) }
                        input type = "password" minlength = "10" required = "";
                        p.error {}
                    }
                    input.button.blue.hover type = "submit" style = "margin: 15px auto 0px;" value=(tr("settings-change-password.dialog-submit"));
                }
            }
        }
    }
}

fn delete_account_dialog() -> Markup {
    html! {
        div.overlay.closable {
            div.dialog #delete-acc-dialog {
                span.plus.cross.hover {}
                h2.underlined.pad {
                    (tr("settings-delete-account.dialog-header"))
                }
                p {
                    (tr("settings-delete-account.dialog-info"))
                }
                form.flex.col novalidate = "" {
                    p.info-red.output {}
                    input.button.red.hover type = "submit" style = "margin: 15px auto 0px;" value=(tr("settings-delete-account.dialog-submit"));
                }
            }
        }
    }
}
