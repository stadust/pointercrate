use crate::account::AccountPageTab;
use maud::{html, Markup, PreEscaped};
use pointercrate_core::permission::PermissionsManager;
use pointercrate_user::auth::AuthenticatedUser;
use sqlx::PgConnection;

pub struct ProfileTab;

#[async_trait::async_trait]
impl AccountPageTab for ProfileTab {
    fn should_display_for(&self, _permissions_we_have: u16, _permissions: &PermissionsManager) -> bool {
        true
    }

    fn initialization_script(&self) -> String {
        "/static/user/js/account/profile.js?v=4".into()
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
        &self, authenticated_user: &AuthenticatedUser, permissions: &PermissionsManager, _connection: &mut PgConnection,
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
                            a.button.blue.hover #link-google href="/api/v1/auth/authorize?legacy=true" type = "button" style = "margin: 15px auto 0px;" {
                                "Link Google"
                            };
                        }
                    }
                }
            }
            div.right {
                div.panel.fade {
                    h2.underlined.pad {
                        "Get access token"
                    }
                    p {
                        "Your pointercrate access token allows you, or programs authorized by you, to make API calls on your behalf. Anyone with access to your pointercrate access token has nearly full control over your account. The only thing that's not possible with only an access token is to change your password. Proceed with care!"
                    }
                    form.flex.col.overlined.pad #login-form novalidate = "" style="display: none" {
                        p style = "text-align: center" {
                            "For security reasons, retrieving your access tokens requires you to reenter your password"
                        }
                        p.info-red.output {}
                        span.form-input #login-password {
                            label for = "password" {"Password:"}
                            input required = "" type = "password" name = "password" minlength = "10";
                            p.error {}
                        }
                        input.button.blue.hover type = "submit" style = "margin: 15px auto 0px;" value="Log in";
                    }
                    div.overlined.pad #token-area style = "display: none" {
                        b {"Your access token is:"}
                        textarea #access-token readonly="" style = "resize: none; width: 100%; margin-top: 8px; min-height:75px" {}
                    }
                    a.blue.hover.button #get-token {
                        "Get access token"
                    }
                }
                div.panel.fade {
                    h2.underlined.pad {
                        "Invalidate tokens"
                    }
                    p {
                        "If one of your access tokens ever got leaked, you can invalidate them here. Invalidating will cause all access tokens to your account to stop functioning. This includes the one stored inside the browser currently, meaning you'll have to log in again after this action"
                    }
                    form.flex.col.overlined.pad #invalidate-form novalidate = "" style="display: none" {
                        p style = "text-align: center" {
                            "For security reasons, invalidating your access tokens requires you to reenter your password"
                        }
                        p.info-red.output {}
                        span.form-input #invalidate-auth-password {
                            label for = "password" {"Password:"}
                            input required = "" type = "password" name = "password" minlength = "10";
                            p.error {}
                        }
                        input.button.blue.hover type = "submit" style = "margin: 15px auto 0px;" value="Invalidate";
                    }
                    a.blue.hover.button #invalidate-token {
                        "Invalidate all access tokens"
                    }
                }
            }
            (edit_display_name_dialog())
            (edit_youtube_link_dialog())
            (change_password_dialog())
            (delete_account_dialog(!authenticated_user.is_legacy()))
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
                p {
                    "To make profile related edits, re-entering your password below is required."
                }
                form.flex.col novalidate = "" {
                    p.info-red.output {}
                    p.info-green.output {}
                    span.form-input #edit-dn {
                        label for = "display_name" {"New display name:"}
                        input type = "text" name = "display_name";
                        p.error {}
                    }
                    span.overlined.pad.form-input #auth-dn {
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

fn edit_youtube_link_dialog() -> Markup {
    html! {
        div.overlay.closable {
            div.dialog #edit-yt-dialog {
                span.plus.cross.hover {}
                h2.underlined.pad {
                    "Edit YouTube Channel Link:"
                }
                p {
                    "To make profile related edits, re-entering your password below is required."
                }
                form.flex.col novalidate = "" {
                    p.info-red.output {}
                    p.info-green.output {}
                    span.form-input #edit-yt {
                        label for = "youtube_channel" {"New YouTube link:"}
                        input type = "url" name = "youtube_channel";
                        p.error {}
                    }
                    span.overlined.pad.form-input #auth-yt {
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

fn delete_account_dialog(is_google: bool) -> Markup {
    // TODO: Add an alternative flow for Google authenticated users

    html! {
        div.overlay.closable {
            div.dialog #delete-acc-dialog {
                span.plus.cross.hover {}
                h2.underlined.pad {
                    "Delete Account:"
                }
                p {
                    "To delete your account, please enter your password below. Deletion of your account is irreversible!"
                }
                form.flex.col novalidate = "" {
                    p.info-red.output {}
                    p.info-green.output {}
                    span.form-input #auth-delete {
                        label {"Authenticate:"}
                        input type = "password" minlength = "10" required = "";
                        p.error {}
                    }
                    input.button.red.hover type = "submit" style = "margin: 15px auto 0px;" value="Delete";
                }
            }
        }
    }
}
