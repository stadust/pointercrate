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

pub struct ProfileTab;

#[async_trait::async_trait]
impl AccountPageTab for ProfileTab {
    fn should_display_for(&self, _permissions_we_have: u16, _permissions: &PermissionsManager) -> bool {
        true
    }

    fn initialization_script(&self) -> String {
        "/static/user/js/account/profile.js".into()
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
                (tr("profile"))
            }
            (PreEscaped("&nbsp;&nbsp;"))
            i class = "fa fa-user fa-2x" aria-hidden="true" {}
        }
    }

    async fn content(
        &self, authenticated_user: &AuthenticatedUser<NonMutating>, permissions: &PermissionsManager, _connection: &mut PgConnection,
    ) -> Markup {
        let user = authenticated_user.user();

        let permissions = permissions.bits_to_permissions(user.permissions);
        let permission_string = permissions.iter().map(|perm| tr(perm.text_id())).collect::<Vec<_>>().join(", ");

        let lang = task_lang();

        html! {
            div.left {
                div.panel.fade {
                    h1.underlined.pad {
                        (trp!("profile.header", "username" = user.name()))
                    }
                    div.flex.space.wrap #things {
                        p.info-red.output style = "margin: 10px" {}
                        p.info-green.output style = "margin: 10px" {}
                        span {
                            b {
                                (tr("profile-username")) ": "
                            }
                            (user.name)
                            p {
                                (tr("profile-username.info"))
                            }
                        }
                        span {
                            b {
                                i.fa.fa-pencil-alt.clickable #display-name-pen aria-hidden = "true" {} " " (tr("profile-display-name")) ": "
                            }
                            i #profile-display-name {
                                @match user.display_name {
                                    Some(ref dn) => (dn),
                                    None => "-"
                                }
                            }
                            p {
                                (tr("profile-display-name.info"))
                            }
                        }
                        span {
                            b {
                                i.fa.fa-pencil-alt.clickable #youtube-pen aria-hidden = "true" {} " " (tr("profile-youtube")) ": "
                            }
                            i #profile-youtube-channel {
                                @match user.youtube_channel {
                                    Some(ref yc) => a.link href = (yc) {},
                                    None => "-"
                                }
                            }
                            p {
                                (tr("profile-youtube.info"))
                            }
                        }
                        span {
                            b {
                                (tr("profile-permissions")) ": "
                            }
                            (permission_string)
                            p {
                                (tr("profile-permissions.info"))
                            }
                        }
                    }
                    div.flex.no-stretch {
                        input.button.red.hover #delete-account type = "button" style = "margin: 15px auto 0px;" value=(tr("profile-delete-account"));
                        @if authenticated_user.is_legacy() {
                            input.button.blue.hover #change-password type = "button" style = "margin: 15px auto 0px;" value=(tr("profile-change-password"));
                        }
                    }
                }
            }
            div.right {
                div.panel.fade {
                    h2.underlined.pad {
                        (tr("profile-logout"))
                    }
                    p {
                        (tr("profile-logout.info"))
                    }
                    a.red.hover.button href = "/logout" style = "margin: 15px auto 0px; display: inline-block" {
                        (tr("profile-logout.button"))
                    }
                }
                @if cfg!(feature = "oauth2") && authenticated_user.is_legacy() {
                    div.panel.fade {
                        h2.underlined.pad {
                            (tr("profile-oauth"))
                        }
                        p {
                            (tr("profile-oauth.info"))
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
                        (tr("profile-get-token"))
                    }
                    p {
                        (tr("profile-get-token.info"))
                    }
                    div.overlined.pad #token-area style = "display: none" {
                        b {(tr("profile-get-token.view-header")) }
                        textarea #access-token readonly="" style = "resize: none; width: 100%; margin-top: 8px; min-height:75px" {}
                    }
                    form.flex.col #get-token-form novalidate = "" {
                        p.info-red.output {}
                        input.blue.hover.button type = "submit" style = "margin: 15px auto 0px;" value=(tr("profile-get-token.button"));
                    }
                }
                div.panel.fade {
                    h2.underlined.pad {
                        (tr("profile-invalidate-tokens"))
                    }
                    p {
                        (tr("profile-invalidate-tokens.info"))
                    }
                    form.flex.col #invalidate-form novalidate = "" {
                        p.info-red.output {}
                        input.blue.hover.button type = "submit" style = "margin: 15px auto 0px;" value=(tr("profile-invalidate-tokens.button"));
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
                    (tr("profile-display-name.dialog-header"))
                }
                form.flex.col novalidate = "" {
                    p.info-red.output {}
                    p.info-green.output {}
                    span.form-input #edit-dn {
                        label for = "display_name" {(tr("profile-display-name.dialog-newname")) }
                        input type = "text" name = "display_name";
                        p.error {}
                    }
                    input.button.blue.hover type = "submit" style = "margin: 15px auto 0px;" value=(tr("profile-display-name.dialog-submit"));
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
                    (tr("profile-youtube.dialog-header"))
                }
                form.flex.col novalidate = "" {
                    p.info-red.output {}
                    p.info-green.output {}
                    span.form-input #edit-yt {
                        label for = "youtube_channel" {(tr("profile-youtube.dialog-newlink")) }
                        input type = "url" name = "youtube_channel";
                        p.error {}
                    }
                    input.button.blue.hover type = "submit" style = "margin: 15px auto 0px;" value=(tr("profile-youtube.dialog-submit"));
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
                    (tr("profile-change-password.dialog-header"))
                }
                p {
                    (tr("profile-change-password.dialog-info"))
                }
                form.flex.col novalidate = "" {
                    p.info-red.output {}
                    p.info-green.output {}
                    span.form-input #edit-pw {
                        label for = "password" {(tr("profile-change-password.dialog-newpassword")) }
                        input type = "password" name = "password" minlength = "10";
                        p.error {}
                    }
                    span.form-input #edit-pw-repeat {
                        label for = "password2" {(tr("profile-change-password.dialog-repeatnewpassword")) }
                        input type = "password"  minlength = "10";
                        p.error {}
                    }
                    span.overlined.pad.form-input #auth-pw {
                        label {(tr("profile-change-password.dialog-authenticate")) }
                        input type = "password" minlength = "10" required = "";
                        p.error {}
                    }
                    input.button.blue.hover type = "submit" style = "margin: 15px auto 0px;" value=(tr("profile-change-password.dialog-submit"));
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
                    (tr("profile-delete-account.dialog-header"))
                }
                p {
                    (tr("profile-delete-account.dialog-info"))
                }
                form.flex.col novalidate = "" {
                    p.info-red.output {}
                    input.button.red.hover type = "submit" style = "margin: 15px auto 0px;" value=(tr("profile-delete-account.dialog-submit"));
                }
            }
        }
    }
}
