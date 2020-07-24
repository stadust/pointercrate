use super::Page;
use crate::{
    extractor::auth::TokenAuth,
    model::{nationality::Nationality, user::User},
    permissions::Permissions,
    state::PointercrateState,
    view::demonlist::{overview_demons, OverviewDemon},
    ApiResult, ViewResult,
};
use actix_web::HttpResponse;
use actix_web_codegen::get;
use maud::{html, Markup, PreEscaped};
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

mod demons;
mod players;
mod profile;
mod records;
mod submitters;
mod users;

#[derive(Debug)]
pub struct AccountPage {
    user: User,
    csrf_token: String,
    demons: Vec<OverviewDemon>,
    pub nations: Vec<Nationality>,
}

#[get("/account/")]
pub async fn index(user: ApiResult<TokenAuth>, state: PointercrateState) -> ViewResult<HttpResponse> {
    Ok(match user {
        Ok(TokenAuth(user)) => {
            let csrf_token = user.generate_csrf_token(&state.secret);

            let (demons, nations) = if user.inner().has_permission(Permissions::ListHelper) {
                let mut connection = state.connection().await?;
                (
                    overview_demons(&mut connection).await?,
                    if user.inner().has_permission(Permissions::ListModerator) {
                        Nationality::all(&mut connection).await?
                    } else {
                        Vec::new()
                    },
                )
            } else {
                (Vec::new(), Vec::new())
            };

            HttpResponse::Ok().content_type("text/html; charset=utf-8").body(
                AccountPage {
                    user: user.into_inner(),
                    csrf_token,
                    demons,
                    nations,
                }
                .render()
                .0,
            )
        },
        Err(_) =>
            actix_web::HttpResponse::Found()
                .header(actix_web::http::header::LOCATION, "/login/")
                .finish(),
    })
}

impl Page for AccountPage {
    fn title(&self) -> String {
        format!("Account - {}", self.user.name)
    }

    fn description(&self) -> String {
        String::new()
    }

    fn scripts(&self) -> Vec<&str> {
        vec![
            "js/modules/form.mjs",
            "js/modules/tab.mjs",
            "js/account/profile.js",
            "js/account/users.js",
            "js/account/records.js",
            "js/account/demon.js",
            "js/account/player.js",
            "js/account/submitter.js",
            "js/staff.js",
        ]
    }

    fn stylesheets(&self) -> Vec<&str> {
        vec!["css/account.css", "css/sidebar.css"]
    }

    fn body(&self) -> Markup {
        html! {
            span#chicken-salad-red-fish style = "display:none" {(self.csrf_token)}
            div.tab-display#account-tabber {
                div.tab-selection.flex.wrap.m-center.fade style="text-align: center;" {
                    div.tab.tab-active.button.white.hover.no-shadow data-tab-id="1" {
                        b {
                            "Profile"
                        }
                        (PreEscaped("&nbsp;&nbsp;"))
                        i class = "fa fa-user fa-2x" aria-hidden="true" {}
                    }
                    @if self.user.has_permission(Permissions::Administrator) || self.user.has_permission(Permissions::ListAdministrator) {
                        div.tab.button.white.hover.no-shadow data-tab-id="2" {
                            b {
                                "Users"
                            }
                            (PreEscaped("&nbsp;&nbsp;"))
                            i class = "fa fa-users fa-2x" aria-hidden="true" {}
                        }
                    }
                    @if self.user.has_permission(Permissions::ListHelper) {
                        div.tab.button.white.hover.no-shadow data-tab-id="3" {
                            b {
                                "Records"
                            }
                            (PreEscaped("&nbsp;&nbsp;"))
                            i class = "fa fa-trophy fa-2x" aria-hidden="true" {}
                        }
                    }
                    @if self.user.has_permission(Permissions::ListModerator) {
                        div.tab.button.white.hover.no-shadow data-tab-id="4" {
                            b {
                                "Players"
                            }
                            (PreEscaped("&nbsp;&nbsp;"))
                            i class = "fa fa-beer fa-2x" aria-hidden="true" {}
                        }
                        div.tab.button.white.hover.no-shadow data-tab-id="5" {
                            i class = "fa fa-shower fa-2x" aria-hidden="true" {}
                            (PreEscaped("&nbsp;&nbsp;"))
                            b {
                                "Demons"
                            }
                        }
                        div.tab.button.white.hover.no-shadow data-tab-id="6" {
                            b {
                                "Submitters"
                            }
                            (PreEscaped("&nbsp;&nbsp;"))
                            i class = "fa fa-eye fa-2x" aria-hidden="true" {}
                        }
                    }
                }

                (profile::page(&self.user))
                @if self.user.has_permission(Permissions::Administrator) || self.user.has_permission(Permissions::ListAdministrator) {
                    (users::page(self.user.has_permission(Permissions::Administrator)))
                }
                @if self.user.has_permission(Permissions::ListHelper) {
                    (records::page(&self.demons))
                }
                @if self.user.has_permission(Permissions::ListModerator) {
                    (players::page(&self.nations))
                    (demons::page())
                    (submitters::page())
                }
            }
        }
    }

    fn head(&self) -> Vec<Markup> {
        let mut hasher = DefaultHasher::new();
        self.user.hash(&mut hasher);

        vec![html! {
            (PreEscaped(
                format!(r#"
                <link href="https://cdnjs.cloudflare.com/ajax/libs/flag-icon-css/3.4.3/css/flag-icon.min.css" rel="stylesheet"><script>window.username='{}'; window.etag='{}'; window.permissions='{}'</script>"#, self.user.name, hasher.finish().to_string(), self.user.permissions.bits())
            ))
        }]
    }
}
