use super::Page;
use crate::state::PointercrateState;
use actix_web::{HttpRequest, Responder};
use maud::{html, Markup};

#[derive(Debug)]
pub struct LoginPage;

pub fn handler(req: &HttpRequest<PointercrateState>) -> impl Responder {
    LoginPage.render(req)
}

//pub fn login(req: &HttpRequest<PointercrateState>) -> impl Responder {}

impl Page for LoginPage {
    fn title(&self) -> String {
        "Pointercrate - Login".to_string()
    }

    fn description(&self) -> String {
        "Log in to an existing pointercrate account or register for a new one!".to_string()
    }

    fn scripts(&self) -> Vec<&str> {
        vec!["js/login.js"]
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
                    div.rightlined.flex.col style="margin: 0" {
                        h2 {"Login"}
                        p {
                            "Log in to an existing pointercrate account. If you do not have an account yet, register on the right or below. If you aren't pointercrate staff, this isn't interesting to you (yet)"
                        }
                        form.flex.col.grow#login-form novalidate = "" {
                            span#login-username {
                                label for = "username" {"Username:"}
                                input required = "" type = "text" name = "username" minlength = "3";
                                p.error {}
                            }
                            span#login-password {
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
                                "Create a new pointercrate account. The chosen username will " i{"not"} " be changable after (not even by asking an admin nicely). "
                            }
                            span#register-username {
                                label for = "username" {"Username:"}
                                input required = "" type = "text" name = "username";
                                p.error {}
                            }
                            span#register-password {
                                label for = "password" {"Password:"}
                                input required = "" type = "password" name = "password" minlength = "10";
                                p.error {}
                            }
                            span#register-password-repeat {
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
