use maud::{html, Markup};
use pointercrate_core_pages::{head::HeadLike, PageFragment};

pub fn login_page() -> PageFragment {
    PageFragment::new(
        "Pointercrate - Login",
        "Log in to an existing pointercrate account or register for a new one!",
    )
    .module("/static/user/js/login.js")
    .module("/static/core/js/modules/form.js")
    .module("/static/core/js/modules/tab.js")
    .stylesheet("/static/user/css/login.css")
    .body(login_page_body())
}

fn login_page_body() -> Markup {
    html! {
        div.tab-display.center #login-tabber style="display: flex; align-items: center; justify-content: center; height: calc(100% - 70px)" { // 70px = height of nav bar
            div.tab-content.tab-content-active.flex.col data-tab-id="1" style="align-items: center" {
                div.panel.fade {
                    h1.underlined.pad {
                        "Sign In"
                    }

                    p {
                        "Sign in using your username and password. Sign in attempts are limited to 3 per 30 minutes."
                    }

                    form.flex.col #login-form novalidate = "" {
                        p.info-red.output {}
                        span.form-input #login-username {
                            label for = "username" {"Username:"}
                            input required = "" type = "text" name = "username" minlength = "3";
                            p.error {}
                        }
                        span.form-input #login-password {
                            label for = "password" {"Password:"}
                            input required = "" type = "password" name = "password" minlength = "10";
                            p.error {}
                        }
                        input.button.blue.hover type = "submit" style = "margin: 15px auto 0px;" value="Sign In";
                    }
                }
                p {
                    "Don't have a pointercrate account yet? " a.link.tab data-tab-id="2" {"Sign up"} " for one!"
                }
            }
            div.tab-content.flex.col data-tab-id="2" style="align-items: center" {
                div.panel.fade {
                    h1.underlined.pad {
                        "Sign Up"
                    }
                    @if cfg!(feature = "legacy_accounts") {
                        p {
                            "Create a new account. Please note that the username cannot be changed after account creation, so choose wisely!"
                        }

                        form.flex.col #register-form novalidate = "" {
                            p.info-red.output {}
                            span.form-input #register-username {
                                label for = "name" {"Username:"}
                                input required = "" type = "text" name = "name";
                                p.error {}
                            }
                            span.form-input #register-password {
                                label for = "password" {"Password:"}
                                input required = "" type = "password" name = "password" minlength = "10";
                                p.error {}
                            }
                            span.form-input #register-password-repeat {
                                label for = "password2" {"Repeat Password:"}
                                input required = "" type = "password" name = "password2" minlength = "10";
                                p.error {}
                            }
                            input.button.blue.hover type = "submit" style = "margin: 15px auto 0px;" value = "Sign Up";
                        }
                    }
                }
                p {
                    "Already have a pointercrate account? " a.link.tab.tab-active data-tab-id="1" {"Sign in"} " instead."
                }
            }
        }
    }
}
