use maud::{html, Markup};
use pointercrate_core_pages::{head::HeadLike, PageFragment};
use pointercrate_user::config;

pub fn login_page() -> PageFragment {
    let mut frag = PageFragment::new(
        "Pointercrate - Login",
        "Log in to an existing pointercrate account or register for a new one!",
    )
    .module("/static/user/js/login.js")
    .module("/static/core/js/modules/form.js")
    .module("/static/core/js/modules/tab.js")
    .stylesheet("/static/user/css/login.css")
    .body(login_page_body());

    if cfg!(feature = "oauth2") {
        frag = frag.async_script("https://accounts.google.com/gsi/client");
    }

    frag
}

fn login_page_body() -> Markup {
    html! {
        div.tab-display.center #login-tabber style="display: flex; align-items: center; justify-content: center; height: calc(100% - 70px)" { // 70px = height of nav bar
            div.tab-content.tab-content-active.flex.col data-tab-id="1" style="align-items: center" {
                div.panel.fade {
                    h1.underlined.pad {
                        "Sign In"
                    }

                    @if cfg!(feature = "oauth2") {
                        p {
                            "If you have linked your pointercrate account with a Google account, you must sign in via Google oauth by clicking the button below:"
                        }
                        div #g_id_onload
                            data-ux_mode="popup"
                            data-auto_select="true"
                            data-itp_support="true"
                            data-client_id=(config::google_client_id())
                            data-callback="googleOauthCallback" {}

                        div .g_id_signin data-text="continue_with" style="margin: 10px 0px" {}
                        p.error #g-signin-error style="text-align: left" {}

                        p.or style="text-size: small; margin: 0px" {"otherwise"}
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
                        input.button.blue.hover type = "submit" style = "margin-top: 15px" value="Sign In";
                    }
                }
                p style = "text-align: center; padding: 0px 10px" {
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
                            input.button.blue.hover type = "submit" style = "margin-top: 15px" value = "Sign Up";
                        }
                    }
                }
                p style = "text-align: center; padding: 0px 10px" {
                    "Already have a pointercrate account? " a.link.tab.tab-active data-tab-id="1" {"Sign in"} " instead."
                }
            }
        }
    }
}
