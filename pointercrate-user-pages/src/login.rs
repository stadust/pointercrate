use maud::{html, Markup, PreEscaped};
use pointercrate_core::{
    localization::{task_lang, tr},
    trp,
};
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
    let lang = task_lang();

    html! {
        div.center #login style="display: flex; align-items: center; justify-content: center; height: calc(100% - 70px)" { // 70px = height of nav bar
            div.flex.col style="align-items: center" {
                div.panel.fade {
                    h1.underlined.pad {
                        (tr("login"))
                    }

                    @if cfg!(feature = "oauth2") {
                        p {
                            (tr("login.oauth-info"))
                        }
                        div #g_id_onload
                            data-ux_mode="popup"
                            data-auto_select="true"
                            data-itp_support="true"
                            data-client_id=(config::google_client_id())
                            data-callback="googleOauthCallback" {}

                        script src=(format!("https://accounts.google.com/gsi/client?hl={}", &lang)) async {}
                        div .g_id_signin data-text="continue_with" style="margin: 10px 0px" data-locale=(lang) {}
                        p.error #g-signin-error style="text-align: left" {}

                        p.or style="text-size: small; margin: 0px" { (tr("login.methods-separator")) }
                    }

                    p {
                        (tr("login.info"))
                    }

                    form.flex.col #login-form novalidate = "" {
                        p.info-red.output {}
                        span.form-input #login-username {
                            label for = "username" {(tr("auth-username")) }
                            input required = "" type = "text" name = "username" minlength = "3";
                            p.error {}
                        }
                        span.form-input #login-password {
                            label for = "password" {(tr("auth-password")) }
                            input required = "" type = "password" name = "password" minlength = "10";
                            p.error {}
                        }
                        input.button.blue.hover type = "submit" style = "margin: 15px auto 0px;" value = (tr("login.submit"));
                    }
                }
                p style = "text-align: center; padding: 0px 10px" {
                    (PreEscaped(trp!(
                        "register.redirect",
                        "redirect-link" =  html! {
                            a.link href="/register" { (tr("register.redirect-link")) }
                        }.into_string()
                    )))
                }
            }
        }
    }
}
