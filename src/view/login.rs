use super::Page;
use crate::{
    api::PCResponder,
    middleware::auth::{Basic, Token},
    state::PointercrateState,
};
use actix_web::{http::Cookie, AsyncResponder, HttpRequest, HttpResponse, Responder};
use cookie::SameSite;
use log::info;
use maud::{html, Markup};
use tokio::prelude::Future;

#[derive(Debug)]
pub struct LoginPage;

pub fn handler(req: &HttpRequest<PointercrateState>) -> PCResponder {
    info!("GET /login/");

    let req_clone = req.clone();

    req.state()
        .auth::<Token>(req.extensions_mut().remove().unwrap())
        .map(move |_| {
            actix_web::HttpResponse::Found()
                .header(actix_web::http::header::LOCATION, "/account/")
                .finish()
        })
        .or_else(move |_| Ok(LoginPage.render(&req_clone).respond_to(&req_clone).unwrap()))
        .responder()
}

/// Alternate login handler for the web interface. Unlike the one in the api, it doesn't return your
/// token, but puts it into a secure, http-only cookie
pub fn login(req: &HttpRequest<PointercrateState>) -> PCResponder {
    info!("POST /login/");

    req.state()
        .auth::<Basic>(req.extensions_mut().remove().unwrap())
        .map(|user| {
            let mut cookie = Cookie::build("access_token", user.0.generate_token())
                .http_only(true)
                .same_site(SameSite::Strict)
                .path("/");

            // allow cookies of HTTP if we're in a debug build, because I dont have a ssl cert for
            // 127.0.0.1 on my laptop smh
            if !cfg!(debug_assertions) {
                cookie = cookie.secure(true)
            }

            HttpResponse::NoContent().cookie(cookie.finish()).finish()
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
                            "Log in to an existing pointercrate account. You have 3 login attempts by 30 minutes. If you do not have an account yet, register on the right or below. "
                        }
                        form.flex.col.grow#login-form novalidate = "" {
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
                            div.grow {}
                            input.button.blue.hover.slightly-round type = "submit" style = "margin: 15px auto 0px;" value="Log in";
                        }
                    }
                    div.flex.col {
                        h2 {"Register"}
                        p {
                            "Not registered yet? Create a new pointercrate account below."
                        }
                        form.flex.col.grow#register-form novalidate = "" {
                            p.info-red.output {}
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
                            input.button.blue.hover.slightly-round type = "submit" style = "margin: 15px auto 0px;" value = "Register";
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
