use super::Page;
use crate::{
    actor::database::TokenAuth, api::PCResponder, model::user::User, state::PointercrateState,
};
use actix_web::{AsyncResponder, HttpRequest, Responder};
use log::info;
use maud::{html, Markup, PreEscaped};
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
        .responder()
}

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
            div.tabbed {
                div.tab-selection.flex.wrap.m-center.fade style="text-align: center;" {
                    div.tab.tab-active.button.white.hover.no-shadow.active data-tab-id="1" {
                        b {
                            "Profile"
                        }
                        (PreEscaped("&nbsp;"))
                        i class = "fa fa-user fa-2x" aria-hidden="true" {}
                    }
                }

                div.tab-display {
                    div.m-center.flex.tab-content-active#container data-tab-id = "1"{
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
                                            Some(ref yc) => (yc),
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
                        }
                        div.right {
                            div.panel.fade {
                                h2.underlined.pad {
                                    "Get access token"
                                }
                                p {
                                    "Your pointercrate access token allows you, or programs authorized by you, to make API calls on your behalf. Anyone with access to your pointercrate access token has nearly full control over your account. The only thing that's not possible with only an access token is to change your password. Proceed with care!"
                                }
                                form.flex.col.grow.underlined.overlined.pad#login-form novalidate = "" style = "text-align: left; margin: 10px 0px;display: none" {
                                    p.info-red.output {}
                                    span.form-input#login-username {
                                        label for = "username" {"Username:"}
                                        input required = "" type = "text" name = "username" minlength = "3";
                                        p.error {}
                                    }
                                    span.form-input#login-password {
                                        label for = "password" {"Password:"}
                                        input required = "" type = "password" name = "password" minlength = "10";
                                        p.error {}
                                    }
                                    input.button.blue.hover type = "submit" style = "margin: 15px auto 0px;" value="Log in";
                                }
                                div.overlined.underlined.pad#token-area style = "display: none" {
                                    b {"You access token is:"}
                                    textarea#access-token readonly="" style = "resize: none; width: 100%; margin-top: 8px" {}
                                }
                                a.blue.hover.button#get-token {
                                    "Get access token"
                                }
                            }
                            div.panel.fade {
                                h2.underlined.pad {
                                    "Edit profile"
                                }
                                p {
                                    "Edit some of the stuff displayed on your profile! You can change your display name and youtube channel link!"
                                }
                                a.blue.hover.button {
                                    "Edit"
                                }
                            }
                        }
                    }
                }

                /*div.m-center.flex.panel.fade.col.wrap style = "margin: 100px 0px;"{
                    h1.underlined.pad {
                        "Under Construction"
                    }

                    div.tabbed.flex {
                        div.tab-selection.flex.col.rightlined style="text-align: center;flex-grow:0"{
                            div.tab.tab-active.hover.scale data-tab-id="1" style="padding: 10px; flex-grow: 0" {
                                h3 {
                                    "Profile"
                                }
                                i class = "fa fa-user fa-2x" aria-hidden="true" {}
                            }
                        }
                        div.tab-display {
                            div.tab-content.tab-content-active data-tab-id ="1" {
                                h2 {
                                    "Profile - " (self.user.name())
                                }

                            div#get-token {
                                p {
                                    "To get a copy of your access token, please reenter your account credentials:"
                                }
                                p#access-token {}
                                form.flex.col.grow#login-form novalidate = "" {
                                    span.form-input#login-username {
                                        label for = "username" {"Username:"}
                                        input required = "" type = "text" name = "username" minlength = "3";
                                        p.error {}
                                    }
                                    span.form-input#login-password {
                                        label for = "password" {"Password:"}
                                        input required = "" type = "password" name = "password" minlength = "10";
                                        p.error {}
                                    }
                                    div.grow {}
                                    input.button.blue.hover type = "submit" style = "margin: 15px auto 0px;" value="Get access token";
                                }
                                div.flex style = "justify-content: end" {
                                    a.blue.hover.button#token {
                                        "Get access token"
                                    }
                                    a.blue.hover.button#edit {
                                        "Edit"
                                    }
                                }
                            }
                        }
                    }
                }*/
            }
        }
    }

    fn head(&self, _: &HttpRequest<PointercrateState>) -> Vec<Markup> {
        vec![]
    }
}
