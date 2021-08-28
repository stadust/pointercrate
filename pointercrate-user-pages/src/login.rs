use maud::{html, Markup};
use pointercrate_core_pages::{PageFragment, Script};

pub struct LoginPage;

impl PageFragment for LoginPage {
    fn title(&self) -> String {
        "Pointercrate - Login".to_string()
    }

    fn description(&self) -> String {
        "Log in to an existing pointercrate account or register for a new one!".to_string()
    }

    fn additional_scripts(&self) -> Vec<Script> {
        vec![
            Script::module("/static/js/login.js"),
            Script::module("/static/js/modules/formv2.js"),
        ]
    }

    fn additional_stylesheets(&self) -> Vec<String> {
        vec!["/static/css/login.css".to_string()]
    }

    fn head_fragment(&self) -> Markup {
        html! {}
    }

    fn body_fragment(&self) -> Markup {
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
                            input.button.blue.hover type = "submit" style = "margin: 15px auto 0px;" value="Log in";
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
                                label for = "name" {"Username:"}
                                input required = "" type = "text" name = "name";
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
}
