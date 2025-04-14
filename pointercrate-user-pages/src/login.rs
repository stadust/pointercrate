use maud::{html, Markup, PreEscaped};
use pointercrate_core::localization::{tr, LANGUAGE};
use pointercrate_core_pages::{head::HeadLike, PageFragment};
use unic_langid::LanguageIdentifier;

pub async fn login_page(lang: &'static LanguageIdentifier) -> PageFragment {
    LANGUAGE
        .scope(lang, async {
            PageFragment::new(
                "Pointercrate - Login",
                "Log in to an existing pointercrate account or register for a new one!",
            )
            .module("/static/user/js/login.js")
            .module("/static/core/js/modules/form.js")
            .module("/static/core/js/modules/tab.js")
            .stylesheet("/static/user/css/login.css")
            .body(login_page_body())
        })
        .await
}

fn login_page_body() -> Markup {
    html! {
        div.tab-display.center #login-tabber style="display: flex; align-items: center; justify-content: center; height: calc(100% - 70px)" { // 70px = height of nav bar
            div.tab-content.tab-content-active data-tab-id="1" {
                div.panel.fade {
                    h1.underlined.pad {
                        (tr("login"))
                    }

                    p {
                        (tr("login.info"))
                    }

                    form.flex.col #login-form novalidate = "" {
                        p.info-red.output {}
                        span.form-input #login-username {
                            label for = "username" {(tr("auth-username")) ":"}
                            input required = "" type = "text" name = "username" minlength = "3";
                            p.error {}
                        }
                        span.form-input #login-password {
                            label for = "password" {(tr("auth-password")) ":"}
                            input required = "" type = "password" name = "password" minlength = "10";
                            p.error {}
                        }
                        input.button.blue.hover type = "submit" style = "margin: 15px auto 0px;" value = (tr("login.submit"));
                    }
                }
                p style = "text-align: center; padding: 0px 10px" {
                    (PreEscaped(tr("register.redirect")))
                }
            }
            div.tab-content data-tab-id="2" {
                div.panel.fade {
                    h1.underlined.pad {
                        (tr("register"))
                    }
                    @if cfg!(feature = "legacy_accounts") {
                        p {
                            (tr("register.info"))
                        }

                        form.flex.col #register-form novalidate = "" {
                            p.info-red.output {}
                            span.form-input #register-username {
                                label for = "name" {(tr("auth-username")) ":"}
                                input required = "" type = "text" name = "name";
                                p.error {}
                            }
                            span.form-input #register-password {
                                label for = "password" {(tr("auth-password")) ":"}
                                input required = "" type = "password" name = "password" minlength = "10";
                                p.error {}
                            }
                            span.form-input #register-password-repeat {
                                label for = "password2" {(tr("auth-repeatpassword")) ":"}
                                input required = "" type = "password" name = "password2" minlength = "10";
                                p.error {}
                            }
                            input.button.blue.hover type = "submit" style = "margin: 15px auto 0px;" value = (tr("register.submit"));
                        }
                    }
                }
                p style = "text-align: center; padding: 0px 10px" {
                    (PreEscaped(tr("login.redirect")))
                }
            }
        }
    }
}
