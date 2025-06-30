use maud::{html, Markup};
use pointercrate_core_pages::head::HeadLike;
use pointercrate_core_pages::PageFragment;
use pointercrate_user::config;

pub fn registration_page() -> PageFragment {
    let mut frag = PageFragment::new("Pointercrate - Registration", "Register for a new pointercrate account!")
        .module("/static/user/js/register.js")
        .module("/static/core/js/modules/form.js")
        .module("/static/core/js/modules/tab.js")
        .stylesheet("/static/user/css/login.css")
        .body(register_page_body());

    if cfg!(feature = "oauth2") {
        frag = frag.async_script("https://accounts.google.com/gsi/client");
    }

    frag
}

fn register_page_body() -> Markup {
    html! {
        div.center #register style="display: flex; align-items: center; justify-content: center; height: calc(100% - 70px)" { // 70px = height of nav bar
            div.flex.col style="align-items: center" {
                div.panel.fade {
                    h1.underlined.pad {
                        "Sign Up"
                    }
                    p {
                        "Create a new account. Please note that the username cannot be changed after account creation, so choose wisely!"
                    }
                    @if cfg!(feature = "oauth2") {
                        div #g_id_onload
                            data-ux_mode="popup"
                            data-auto_select="true"
                            data-itp_support="true"
                            data-client_id=(config::google_client_id())
                            data-callback="googleOauthRegisterCallback" {}

                        div .g_id_signin data-text="signup_with" style="margin: 10px 0px" {}
                        @if cfg!(feature = "legacy_accounts") {
                            p.or style="text-size: small; margin: 0px" {"or"}
                        }
                    }
                    @if cfg!(feature = "legacy_accounts") {
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
                    "Already have a pointercrate account? " a.link href="/login" {"Sign in"} " instead."
                }
            }
        }
        @if cfg!(feature = "oauth2") {
            (oauth_registration_dialog())
        }
    }
}


fn oauth_registration_dialog() -> Markup {
    html! {
        div.overlay.closable {
            div.dialog #oauth-registration-pick-username style="width: 400px" {
                span.plus.cross.hover {}
                h2.underlined.pad {
                    "Pick your username:"
                }
                form.flex.col novalidate = "" {
                    p.info-red.output {}
                    span.form-input #oauth-username {
                        label for = "username" {"Username:"}
                        input type = "text" name = "username";
                        p.error {}
                    }
                    input.button.blue.hover type = "submit" style = "margin: 15px auto 0px;" value="Sign Up!";
                }
            }
        }
    }
}