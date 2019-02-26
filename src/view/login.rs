use super::Page;
use crate::{
    actor::database::BasicAuth, api::PCResponder, model::user::User, state::PointercrateState,
};
use actix_web::{http::Cookie, AsyncResponder, HttpRequest, HttpResponse, Responder};
use cookie::SameSite;
use log::info;
use maud::{html, Markup};
use tokio::prelude::Future;

#[derive(Debug)]
pub struct LoginPage;

pub fn handler(req: &HttpRequest<PointercrateState>) -> impl Responder {
    // TODO: if already logged in, redirect to /account/
    LoginPage.render(req)
}

/// Alternate login handler for the web interface. Unlike the one in the api, it doesn't return your
/// token, but puts it into a secure, http-only cookie
pub fn login(req: &HttpRequest<PointercrateState>) -> PCResponder {
    info!("POST /login/");

    req.state()
        .database(BasicAuth(req.extensions_mut().remove().unwrap()))
        .map(|user: User| {
            HttpResponse::NoContent()
                .cookie(
                    Cookie::build("access_token", user.generate_token())
                        .http_only(true) // TODO: secure cookies if and only if we have an https connection
                        .same_site(SameSite::Strict)
                        .path("/")
                        .finish(),
                )
                .finish()
        })
        .responder()
}

impl Page for LoginPage {
    fn title(&self) -> String {
        "Pointercrate - Login".to_string()
    }

    fn description(&self) -> String {
        "Log in to an existing pointercrate account or register for a new one!".to_string()
    }

    fn scripts(&self) -> Vec<&str> {
        vec!["js/login.js", "js/form.js"]
    }

    fn stylesheets(&self) -> Vec<&str> {
        vec!["css/login.css"]
    }

    fn body(&self, req: &HttpRequest<PointercrateState>) -> Markup {
        html! {
            div.m-center.flex.panel.fade.col.wrap style = "margin: 100px 0px;"{
                h1.underlined.pad {
                    "Pointercrate Account"
                }
                p {
                    "By using pointercrate accounts you agree to cookies. If you don't then I formally request you to stop using the internet as you obviously have no idea what you're talking about. "
                }
                div.flex#login {
                    div.flex.col {
                        h2 {"Login"}
                        p {
                            "Log in to an existing pointercrate account. If you do not have an account yet, register on the right or below. "
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
                            input.button.blue.hover type = "submit" style = "margin: 15px auto 0px;" value="Log in";
                        }
                    }
                    div.flex.col {
                        h2 {"Register"}
                        form.flex.col.grow#register-form novalidate = "" {
                            p {
                                "Not registered yet? Create a new pointercrate account below."
                            }
                            span.form-input#register-username {
                                label for = "username" {"Username:"}
                                input required = "" type = "text" name = "username";
                                p.error {}
                            }
                            span.form-input#register-password {
                                label for = "password" {"Password:"}
                                input required = "" type = "password" name = "password" minlength = "10";
                                p.error {}
                            }
                            span.form-input#register-password-repeat {
                                label for = "password2" {"Repeat Password:"}
                                input required = "" type = "password" name = "password2" minlength = "10";
                                p.error {}
                            }
                            div.grow {}
                            input.button.blue.hover type = "submit" style = "margin: 15px auto 0px;" value = "Register";
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
