use log::error;
use maud::{html, Markup, PreEscaped};
use pointercrate_core::{
    error::PointercrateError,
    localization::{ftr, tr},
    permission::PermissionsManager,
};
use pointercrate_core_pages::{
    error::ErrorFragment,
    util::{filtered_paginator, paginator},
};
use pointercrate_demonlist::player::claim::PlayerClaim;
use pointercrate_user::{
    auth::{AuthenticatedUser, NonMutating},
    MODERATOR,
};
use pointercrate_user_pages::account::AccountPageTab;
use sqlx::PgConnection;
use unic_langid::LanguageIdentifier;

pub struct ListIntegrationTab(#[doc = "discord invite url"] pub &'static str);

#[async_trait::async_trait]
impl AccountPageTab for ListIntegrationTab {
    fn should_display_for(&self, _permissions_we_have: u16, _permissions: &PermissionsManager) -> bool {
        true
    }

    fn initialization_script(&self) -> String {
        "/static/demonlist/js/account/integration.js".into()
    }

    fn tab_id(&self) -> u8 {
        7
    }

    fn tab(&self, lang: &'static LanguageIdentifier) -> Markup {
        html! {
            b {
                (tr(lang, "list-integration"))
            }
            (PreEscaped("&nbsp;&nbsp;"))
            i class = "fa fa-list fa-2x" aria-hidden="true" {}
        }
    }

    async fn content(
        &self, lang: &'static LanguageIdentifier, user: &AuthenticatedUser<NonMutating>, permissions: &PermissionsManager,
        connection: &mut PgConnection,
    ) -> Markup {
        let player_claim = match PlayerClaim::by_user(user.user().id, connection).await {
            Ok(player_claim) => player_claim,
            Err(err) => {
                error!("Error retrieving player claim of user {}: {:?}", user.user(), err);

                return ErrorFragment {
                    status: err.status_code(),
                    reason: "Internal Server Error".to_string(),
                    message: err.to_string(),
                }
                .body();
            },
        };
        let is_moderator = permissions.require_permission(user.user().permissions, MODERATOR).is_ok();

        html! {
            div.left {
                div.panel.fade.js-collapse style="text-align: left; padding: 10px 20px" {
                    div.flex.no-stretch style="justify-content: space-between; align-items: center; " {
                        span style = "font-size: 1.3em" {
                            i.fa.fa-pencil-alt.clickable #player-claim-pen aria-hidden = "true" {} (PreEscaped("&nbsp;"))
                            b {
                                (tr(lang, "claimed-player")) ": "
                            }
                            @match player_claim {
                                Some(ref claim) => {
                                    i #claimed-player data-id = (claim.player.id) {
                                        (claim.player.name)
                                    }
                                },
                                None => {
                                    i {
                                        "None"
                                    }
                                }
                            }
                        }
                        span {
                            @match player_claim {
                                Some(ref claim) if claim.verified => {
                                    i style="margin-right: 15px;" {
                                        (tr(lang, "claimed-player.verified"))
                                    }
                                    span.arrow.hover {}
                                },
                                Some(_) => i{(tr(lang, "claimed-player.unverified"))},
                                _ => {}
                            }
                        }
                    }
                    @if let Some(ref claim) = player_claim {
                        @if claim.verified {
                            div.overlined.pad.js-collapse-content #claims-claim-panel style="display:none" {
                                p.info-red.output style = "margin: 10px 0" {}
                                p.info-green.output style = "margin: 10px 0" {}
                                div.flex.no-stretch style="justify-content: space-between; align-items: center" {
                                    b {
                                        (tr(lang, "claim-geolocate")) ":"
                                    }
                                    a.button.blue.hover #claims-geolocate-nationality {
                                        (tr(lang, "claim-geolocate.submit"))
                                    }
                                }
                                p {
                                    (PreEscaped(tr(lang, "claim-geolocate.info")))
                                }
                                div.cb-container.flex.no-stretch style="justify-content: space-between; align-items: center" {
                                    b {
                                        (tr(lang, "claim-lock-submissions")) ":"
                                    }
                                    @if claim.lock_submissions {
                                        input #lock-submissions-checkbox type = "checkbox" name = "lock_submissions" checked = "";
                                    }
                                    @else {
                                        input #lock-submissions-checkbox type = "checkbox" name = "lock_submissions";
                                    }
                                    span.checkmark {}
                                }
                                p {
                                    (tr(lang, "claim-lock-submissions.info")) ":"
                                }
                            }
                        }
                    }
                }
                @if let Some(claim) = player_claim {
                    @if claim.verified {
                        div.panel.fade {
                            h2.pad.underlined {
                                (tr(lang, "claim-records"))
                            }
                            p {
                                (PreEscaped(tr(lang, "claim-records.info")))
                            }
                            (paginator("claims-record-pagination", "/api/v1/records/"))
                        }
                    }
                }
                @if is_moderator {
                    div.panel.fade {
                        h2.pad.underlined {
                            (tr(lang, "claim-manager"))
                        }
                        p {
                            (tr(lang, "claim-manager.info-a"))
                            br;
                            (tr(lang, "claim-manager.info-b"))
                            br;
                            (tr(lang, "claim-manager.info-c"))
                            br;
                            (tr(lang, "claim-manager.info-d"))
                        }
                        (filtered_paginator("claim-pagination", "/api/v1/players/claims/"))
                    }
                }
            }
            div.right {
                div.panel.fade style = "display: none;"{
                    h2.underlined.pad {
                        (tr(lang, "claim-initiate-panel"))
                    }
                    p {
                        (tr(lang, "claim-initiate-panel.info"))
                    }
                    (filtered_paginator("claims-initiate-claim-pagination", "/api/v1/players/"))
                }
                div.panel.fade {
                    h2.underlined.pad {
                        (tr(lang, "claim-info-panel"))
                    }
                    p {
                        (tr(lang, "claim-info-panel.info-a"))
                        br;
                        (PreEscaped(ftr(lang, "claim-info-panel.info-b", &vec![
                            (
                                "discord",
                                format!("<a class=\"link\" href=\"{}\">{}</a>", &self.0, tr(lang, "claim-info-panel.info-discord"))
                            )
                        ])))
                        br;
                        (tr(lang, "claim-info-panel.info-c"))
                    }
                }
                @if is_moderator {
                    div.panel.fade {
                        h2.underlined.pad {
                            (tr(lang, "claim-video-panel"))
                        }
                        p {
                            (tr(lang, "claim-video-panel.info"))
                        }
                        iframe."ratio-16-9"#claim-video style="width:100%;" allowfullscreen="" {}
                    }
                }
            }
        }
    }
}
