use super::Page;
use crate::{
    extractor::{
        auth::{BasicAuth, TokenAuth},
        ip::Ip,
    },
    model::user::{AuthenticatedUser, Registration},
    ratelimit::RatelimitScope,
    state::PointercrateState,
    ApiResult,
};
use actix_web::{cookie::SameSite, http::Cookie, web::Json, HttpResponse};
use actix_web_codegen::{get, post};
use maud::{html, Markup};

#[derive(Debug, Copy, Clone)]
pub struct LoginPage;

#[get("/login/")]
pub fn index(user: ApiResult<TokenAuth>) -> HttpResponse {
    match user {
        Ok(user) => HttpResponse::Found().header("Location", "/account/").finish(),
        _ =>
            HttpResponse::Ok()
                .content_type("text/html; charset=utf-8")
                .body(LoginPage.render().0),
    }
}

/// Alternate login handler for the web interface. Unlike the one in the api, it doesn't return your
/// token, but puts it into a secure, http-only cookie
#[post("/login/")]
pub async fn post(Ip(ip): Ip, auth: ApiResult<BasicAuth>, state: PointercrateState) -> ApiResult<HttpResponse> {
    state.ratelimits.check(RatelimitScope::Login, ip)?;

    // we have to explicitly take the Result here and transform it into a ViewResult so that we get a
    // Html error page >.>
    let BasicAuth(user) = auth?;

    let mut cookie = Cookie::build("access_token", user.generate_token(&state.secret))
        .http_only(true)
        .same_site(SameSite::Strict)
        .path("/");

    // allow cookies of HTTP if we're in a debug build, because I don't have a ssl cert for
    // 127.0.0.1 on my laptop smh
    if !cfg!(debug_assertions) {
        cookie = cookie.secure(true)
    }

    Ok(HttpResponse::NoContent().cookie(cookie.finish()).finish())
}

/// Alternate register handler for the web interface. Unlike the one in the api, it doesn't return
/// your token, but puts it into a secure, http-only cookie
#[post("/register/")]
pub async fn register(Ip(ip): Ip, body: Json<Registration>, state: PointercrateState) -> ApiResult<HttpResponse> {
    let mut connection = state.connection().await?;
    let user = AuthenticatedUser::register(body.into_inner(), &mut connection, Some(state.ratelimits.prepare(ip))).await?;

    let mut cookie = Cookie::build("access_token", user.generate_token(&state.secret))
        .http_only(true)
        .same_site(SameSite::Strict)
        .path("/");

    // allow cookies of HTTP if we're in a debug build, because I don't have a ssl cert for
    // 127.0.0.1 on my laptop smh
    if !cfg!(debug_assertions) {
        cookie = cookie.secure(true)
    }

    Ok(HttpResponse::Created().cookie(cookie.finish()).finish())
}

impl Page for LoginPage {
    fn title(&self) -> String {
        "Pointercrate - Login".to_string()
    }

    fn description(&self) -> String {
        "Log in to an existing pointercrate account or register for a new one!".to_string()
    }

    fn scripts(&self) -> Vec<&str> {
        vec!["js/login.js", "js/modules/form.mjs"]
    }

    fn stylesheets(&self) -> Vec<&str> {
        vec!["css/login.css"]
    }

    fn body(&self) -> Markup {
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

    fn head(&self) -> Vec<Markup> {
        vec![]
    }
}
