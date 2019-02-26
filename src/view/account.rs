use super::Page;
use crate::{
    actor::database::TokenAuth, api::PCResponder, model::user::User, state::PointercrateState,
};
use actix_web::{AsyncResponder, HttpRequest, Responder};
use log::info;
use maud::{html, Markup};
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
        vec!["css/account.css"]
    }

    fn body(&self, req: &HttpRequest<PointercrateState>) -> Markup {
        html! {
            div.m-center.flex.panel.fade.col.wrap style = "margin: 100px 0px;"{
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
                        p {
                            "To get a copy of your access token, please reenter your account credentials:"
                        }
                        p#access-token {

                        }
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
        }
    }

    fn head(&self, _: &HttpRequest<PointercrateState>) -> Vec<Markup> {
        vec![]
    }
}
