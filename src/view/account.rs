use super::Page;
use crate::{
    actor::database::TokenAuth, api::PCResponder, model::user::User, state::PointercrateState,
};
use actix_web::{AsyncResponder, HttpRequest, Responder};
use log::info;
use maud::{html, Markup, PreEscaped};
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};
use tokio::prelude::Future;

#[derive(Debug)]
pub struct AccountPage {
    user: User,
}

pub fn handler(req: &HttpRequest<PointercrateState>) -> PCResponder {
    info!("GET /account/");

    let req_clone = req.clone();

    req.state()
        .database(TokenAuth(req.extensions_mut().remove().unwrap()))
        .map(move |user: User| {
            AccountPage { user }
                .render(&req_clone)
                .respond_to(&req_clone)
                .unwrap()
        })
        .or_else(|_| {
            Ok(actix_web::HttpResponse::Found()
                .header(actix_web::http::header::LOCATION, "/login/")
                .finish())
        })
        .responder()
}

// TODO: support to invalidate access tokens

impl Page for AccountPage {
    fn title(&self) -> String {
        format!("Account - {}", self.user.name)
    }

    fn description(&self) -> String {
        String::new()
    }

    fn scripts(&self) -> Vec<&str> {
        vec!["js/account.js", "js/form.js"]
    }

    fn stylesheets(&self) -> Vec<&str> {
        vec!["css/account.css", "css/sidebar.css"]
    }

    fn body(&self, req: &HttpRequest<PointercrateState>) -> Markup {
        html! {
            span#chicken-salad-red-fish style = "display:none" {(self.user.generate_csrf_token())}
            div.tabbed {
                div.tab-selection.flex.wrap.m-center.fade style="text-align: center;" {
                    div.tab.tab-active.button.white.hover.no-shadow data-tab-id="1" {
                        b {
                            "Profile"
                        }
                        (PreEscaped("&nbsp;"))
                        i class = "fa fa-user fa-2x" aria-hidden="true" {}
                    }
                    @if self.user.has_any(&perms!(Administrator)) {
                        div.tab.button.white.hover.no-shadow data-tab-id="2" {
                            b {
                                "Users"
                            }
                            (PreEscaped("&nbsp;"))
                            i class = "fa fa-users fa-2x" aria-hidden="true" {}
                        }
                    }
                }

                div.tab-display {
                    (self.profile_page())
                    (self.user_page())
                }
            }
        }
    }

    fn head(&self, _: &HttpRequest<PointercrateState>) -> Vec<Markup> {
        let mut hasher = DefaultHasher::new();
        self.user.hash(&mut hasher);

        vec![html! {
            (PreEscaped(
                format!("<script>window.username='{}'; window.etag='{}';</script>", self.user.name, hasher.finish().to_string())
            ))
        }]
    }
}

impl AccountPage {
    fn user_page(&self) -> Markup {
        html! {
            div.m-center.flex.tab-content.container data-tab-id = "2" {
                div.left {
                    div.panel.fade {
                        h1.underlined.pad {
                            "User"
                        }
                        p#text {
                            "Use the panels on the right to select users to modify"
                        }
                        form.flex.col.pad#patch-permissions novalidate = "" style="display:none" {
                            h3 {
                                "Permissions:"
                            }
                            p.info-red.output {}
                            p.info-green.output {}
                            label.cb-container.form-input#perm-extended for = "extended" {
                                i{"Extended access"}
                                input type = "checkbox" name = "extended";
                                span.checkmark {}
                            }
                            label.form-input.cb-container#perm-list-helper for = "helper" {
                                i {"List Helper"}
                                input type = "checkbox" name = "helper";
                                span.checkmark {}
                            }
                            label.form-input.cb-container#perm-list-mod for = "mod" {
                                i {"List Moderator"}
                                input type = "checkbox" name = "mod";
                                span.checkmark {}
                            }
                            label.form-input.cb-container#perm-list-admin for = "admin" {
                                i {"List Administrator"}
                                input type = "checkbox" name = "admin";
                                span.checkmark {}
                            }
                            label.form-input.cb-container#perm-mod for = "mod2" {
                                i {"Pointercrate Moderator"}
                                input type = "checkbox" name = "mod2";
                                span.checkmark {}
                            }
                            label.form-input.cb-container#perm-admin for = "admin2" {
                                i {"Pointercrate Administrator"}
                                input type = "checkbox" name = "admin2";
                                span.checkmark {}
                            }
                            div.flex.no-stretch {
                                input.button.blue.hover.slightly-round#delete-user type = "button" style = "margin: 15px auto 0px;" value="Delete user";
                                input.button.blue.hover.slightly-round type = "submit" style = "margin: 15px auto 0px;" value="Edit user";
                            }
                        }
                    }
                }
                div.right {
                    div.panel.fade {
                        h2.underlined.pad {
                            "Find users"
                        }
                        p {
                            "Users can be uniquely identified by name and ID. To modify a user's account, you need their ID. If you know neither, try looking in the list below"
                        }
                        form.flex.col.underlined.pad#find-id-form novalidate = "" {
                            p.info-red.output {}
                            span.form-input#find-id {
                                label for = "id" {"User ID:"}
                                input required = "" type = "number" name = "id" min = "0" style="width:93%"; // FIXME: I have no clue why the input thinks it's a special snowflake and fucks up its width, but I dont have the time to fix it
                                p.error {}
                            }
                            input.button.blue.hover.slightly-round type = "submit" style = "margin: 15px auto 0px;" value="Find by ID";
                        }
                        form.flex.col#find-name-form novalidate = "" {
                            p.info-red.output {}
                            span.form-input#find-name {
                                label for = "name" {"Username:"}
                                input required = "" type = "text" name = "name" minlength = "3";
                                p.error {}
                            }
                            input.button.blue.hover.slightly-round type = "submit" style = "margin: 15px auto 0px;" value="Find by name";
                        }
                    }
                    div.panel.fade {
                        h2.underlined.pad {
                            "User list"
                        }
                        p {
                            "A list of all user accounts on pointercrate"
                        }
                        a.blue.hover.button.slightly-round#find-name {
                            "Load"
                        }
                    }
                }
            }
        }
    }

    fn profile_page(&self) -> Markup {
        html! {
            div.m-center.flex.tab-content.tab-content-active.container data-tab-id = "1"{
                div.left {
                    div.panel.fade {
                        h1.underlined.pad {
                            "Profile - " (self.user.name())
                        }
                        div.flex.space.wrap#things {
                            span {
                                b {
                                    "Username: "
                                }
                                (self.user.name)
                                p {
                                    "The name you registered under and which you use to log in to pointercrate. This name is unique to your account, and cannot be changed"
                                }
                            }
                            span {
                                b {
                                    "Display name: "
                                }
                                @match self.user.display_name {
                                    Some(ref dn) => (dn),
                                    None => "-"
                                }
                                p {
                                    "If set, this name will be displayed instead of your username. Display names aren't unique."
                                }
                            }
                            span {
                                b {
                                    "Youtube channel: "
                                }
                                @match self.user.youtube_channel {
                                    Some(ref yc) => a.link href = (yc) {},
                                    None => "-"
                                }
                                p {
                                    "A link to your YouTube channel, if you have one. If set, all mentions of your name will turn into links to it."
                                }
                            }
                            span {
                                b {
                                    "Permissions: "
                                }
                                (self.user.permissions())
                                p {
                                    "The permissions you have on pointercrate. 'Extended Access' means you can retrieve more data from the API if you authorize yourself, 'List ...' means you're a member of the demonlist team. 'Moderator'  and 'Administrator' mean you're part of pointercrate's staff team."
                                }
                            }
                        }
                    }
                    div.panel.fade.closable#edit style = "display: none" {
                        span.plus.cross.hover {}
                        h2.underlined.pad {
                            "Edit profile"
                        }
                        p {
                            "Modifying your account requires you to re-authenticate using your password. " i{"Changing"} " your password will log you out and redirect to the login page. It will further invalidate all access tokens to your account"
                        }
                        form.flex.col#edit-form novalidate = "" {
                            p.info-red.output {}
                            span.form-input#edit-display-name {
                                label for = "display_name" {"New display name:"}
                                input type = "text" name = "username" value = (self.user.display_name.as_ref().map(AsRef::as_ref).unwrap_or(""));
                                p.error {}
                            }
                            span.form-input#edit-yt-channel {
                                label for = "yt_channel" {"New YouTube channel:"}
                                input type = "url" name = "yt_channel" value = (self.user.youtube_channel.as_ref().map(AsRef::as_ref).unwrap_or(""));
                                p.error {}
                            }
                            span.form-input#edit-password {
                                label for = "password" {"New password:"}
                                input type = "password" name = "password" minlength = "10";
                                p.error {}
                            }
                            span.form-input#edit-password-repeat {
                                label for = "password2" {"Repeat new password:"}
                                input type = "password" name = "password2" minlength = "10";
                                p.error {}
                            }
                            span.overlined.underlined.pad.form-input#auth-password {
                                label for = "auth-password" {"Authenticate:"}
                                input type = "password" name ="auth-password" minlength = "10" required = "";
                                p.error {}
                            }
                            input.button.blue.hover.slightly-round type = "submit" style = "margin: 15px auto 0px;" value="Submit edit";
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
                        form.flex.col.overlined.pad#login-form novalidate = "" style="display: none" {
                            p style = "text-align: center" {
                                "For security reasons, retrieving your access tokens requires you to reenter your password"
                            }
                            p.info-red.output {}
                            span.form-input#login-password {
                                label for = "password" {"Password:"}
                                input required = "" type = "password" name = "password" minlength = "10";
                                p.error {}
                            }
                            input.button.blue.hover.slightly-round type = "submit" style = "margin: 15px auto 0px;" value="Log in";
                        }
                        div.overlined.pad#token-area style = "display: none" {
                            b {"Your access token is:"}
                            textarea#access-token readonly="" style = "resize: none; width: 100%; margin-top: 8px; min-height:75px" {}
                        }
                        a.blue.hover.button.slightly-round#get-token {
                            "Get access token"
                        }
                    }
                    div.panel.fade {
                        h2.underlined.pad {
                            "Edit profile"
                        }
                        p {
                            "Edit some of the stuff displayed on your profile! You can change your display name and youtube channel link! You can also change your password here"
                        }
                        a.blue.hover.button.slightly-round.js-scroll data-destination = "edit" data-reveal = "true" {
                            "Edit"
                        }
                    }
                    div.panel.fade {
                        h2.underlined.pad {
                            "Invalidate tokens"
                        }
                        p {
                            "If one of your access tokens ever got leaked, you can invalidate them here. Invalidating will cause all access tokens to your account to stop functioning. This includes the one stored inside the browser currently, meaning you'll have to log in again after this action"
                        }
                        form.flex.col.overlined.pad#invalidate-form novalidate = "" style="display: none" {
                            p style = "text-align: center" {
                                "For security reasons, invalidating your access tokens requires you to reenter your password"
                            }
                            p.info-red.output {}
                            span.form-input#invalidate-auth-password {
                                label for = "password" {"Password:"}
                                input required = "" type = "password" name = "password" minlength = "10";
                                p.error {}
                            }
                            input.button.blue.hover.slightly-round type = "submit" style = "margin: 15px auto 0px;" value="Invalidate";
                        }
                        a.blue.hover.button.slightly-round#invalidate-token {
                            "Invalidate all access tokens"
                        }
                    }
                }
            }
        }
    }
}
