use crate::account::AccountPageTab;
use maud::{html, Markup, PreEscaped};
use pointercrate_core::permission::PermissionsManager;
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
                "Profile"
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
        let permission_string = permissions.iter().map(|perm| perm.name()).collect::<Vec<_>>().join(", ");

        html! {
            div.left {
                div.panel.fade {
                    h1.underlined.pad {
                        "Profile - " (user.name())
                    }
                    div.flex.space.wrap #things {
                        p.info-red.output style = "margin: 10px" {}
                        p.info-green.output style = "margin: 10px" {}
                        span {
                            b {
                                "Username: "
                            }
                            (user.name)
                            p {
                                "The name you registered under and which you use to log in to pointercrate. This name is unique to your account, and cannot be changed"
                            }
                        }
                        span {
                            b {
                                i.fa.fa-pencil-alt.clickable #display-name-pen aria-hidden = "true" {} " Display name: "
                            }
                            i #profile-display-name {
                                @match user.display_name {
                                    Some(ref dn) => (dn),
                                    None => "-"
                                }
                            }
                            p {
                                "If set, this name will be displayed instead of your username. Display names aren't unique and you cannot use your display name to login to your pointercrate account."
                            }
                        }
                        span {
                            b {
                                i.fa.fa-pencil-alt.clickable #youtube-pen aria-hidden = "true" {} " YouTube channel: "
                            }
                            i #profile-youtube-channel {
                                @match user.youtube_channel {
                                    Some(ref yc) => a.link href = (yc) {},
                                    None => "-"
                                }
                            }
                            p {
                                "A link to your YouTube channel, if you have one. If set, all mentions of your name will turn into links to it."
                            }
                        }
                        span {
                            b {
                                "Permissions: "
                            }
                            (permission_string)
                            p {
                                "The permissions you have on pointercrate. 'List ...' means you're a member of the demonlist team. 'Moderator'  and 'Administrator' mean you're part of pointercrate's staff team."
                            }
                        }
                    }
                    div.flex.no-stretch {
                        input.button.red.hover #delete-account type = "button" style = "margin: 15px auto 0px;" value="Delete My Account";
                        @if authenticated_user.is_legacy() {
                            input.button.blue.hover #change-password type = "button" style = "margin: 15px auto 0px;" value="Change Password";
                        }
                    }
                }
            }
            div.right {
                div.panel.fade {
                    h2.underlined.pad {
                        "Logout"
                    }
                    p {
                        "Log out of your pointercrate account in this browser."
                    }
                    a.red.hover.button href = "/logout" style = "margin: 15px auto 0px; display: inline-block" {
                        "Logout"
                    }
                }
                @if cfg!(feature = "oauth2") && authenticated_user.is_legacy() {
                    div.panel.fade {
                        h2.underlined.pad {
                            "Link With Google"
                        }
                        p {
                            "Enable signing in to your pointercrate account via Google oauth. More secure than password login, and avoids account lock-outs due to forgotten passwords. Linking a Google account is irreversible, and you cannot change the linked Google account later on!"
                        }
                        div #g_id_onload
                            data-ux_mode="popup"
                            data-auto_select="true"
                            data-itp_support="true"
                            data-client_id=(config::google_client_id())
                            data-callback="googleOauthCallback" {}

                        div .g_id_signin data-text="continue_with" style="margin: 10px 0px" {}
                        p.error #g-signin-error style="text-align: left" {}
                    }
                }
                div.panel.fade {
                    h2.underlined.pad {
                        "Get access token"
                    }
                    p {
                        "Your pointercrate access token allows you, or programs authorized by you, to make API calls on your behalf. They do not allow modifications of your account however."
                    }
                    div.overlined.pad #token-area style = "display: none" {
                        b {"Your access token is:"}
                        textarea #access-token readonly="" style = "resize: none; width: 100%; margin-top: 8px; min-height:75px" {}
                    }
                    form.flex.col #get-token-form novalidate = "" {
                        p.info-red.output {}
                        input.blue.hover.button type = "submit" style = "margin: 15px auto 0px;" value="Get access token";
                    }
                }
                div.panel.fade {
                    h2.underlined.pad {
                        "Invalidate tokens"
                    }
                    p {
                        "If one of your access tokens ever got leaked, you can invalidate them here. Invalidating will cause all access tokens to your account to stop functioning. This includes the one stored inside the browser currently, meaning you'll have to log in again after this action!"
                    }
                    form.flex.col #invalidate-form novalidate = "" {
                        p.info-red.output {}
                        input.blue.hover.button type = "submit" style = "margin: 15px auto 0px;" value="Invalidate all access tokens";
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
                    "Edit Display Name:"
                }
                form.flex.col novalidate = "" {
                    p.info-red.output {}
                    p.info-green.output {}
                    span.form-input #edit-dn {
                        label for = "display_name" {"New display name:"}
                        input type = "text" name = "display_name";
                        p.error {}
                    }
                    input.button.blue.hover type = "submit" style = "margin: 15px auto 0px;" value="Edit";
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
                    "Edit YouTube Channel Link:"
                }
                form.flex.col novalidate = "" {
                    p.info-red.output {}
                    p.info-green.output {}
                    span.form-input #edit-yt {
                        label for = "youtube_channel" {"New YouTube link:"}
                        input type = "url" name = "youtube_channel";
                        p.error {}
                    }
                    input.button.blue.hover type = "submit" style = "margin: 15px auto 0px;" value="Edit";
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
                    "Change Password:"
                }
                p {
                    "To make profile related edits, re-entering your password below is required. " i{"Changing"} " your password will log you out and redirect to the login page. It will further invalidate all access tokens to your account"
                }
                form.flex.col novalidate = "" {
                    p.info-red.output {}
                    p.info-green.output {}
                    span.form-input #edit-pw {
                        label for = "password" {"New password:"}
                        input type = "password" name = "password" minlength = "10";
                        p.error {}
                    }
                    span.form-input #edit-pw-repeat {
                        label for = "password2" {"Repeat new password:"}
                        input type = "password"  minlength = "10";
                        p.error {}
                    }
                    span.overlined.pad.form-input #auth-pw {
                        label {"Authenticate:"}
                        input type = "password" minlength = "10" required = "";
                        p.error {}
                    }
                    input.button.blue.hover type = "submit" style = "margin: 15px auto 0px;" value="Edit";
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
                    "Delete Account:"
                }
                p {
                    "Deletion of your account is irreversible!"
                }
                form.flex.col novalidate = "" {
                    p.info-red.output {}
                    input.button.red.hover type = "submit" style = "margin: 15px auto 0px;" value="Delete";
                }
            }
        }
    }
}
