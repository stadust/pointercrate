use crate::components::P;
use log::error;
use maud::{html, Markup, PreEscaped};
use pointercrate_core::{error::PointercrateError, localization::tr, permission::PermissionsManager, trp};
use pointercrate_core_pages::{
    error::ErrorFragment,
    trp_html,
    util::{filtered_paginator, paginator},
};
use pointercrate_demonlist::{list::List, player::claim::PlayerClaim};
use pointercrate_user::{
    auth::{AuthenticatedUser, NonMutating},
    MODERATOR,
};
use pointercrate_user_pages::account::AccountPageTab;
use sqlx::PgConnection;

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

    fn tab(&self) -> Markup {
        html! {
            b {
                (tr("list-integration"))
            }
            (PreEscaped("&nbsp;&nbsp;"))
            i class = "fa fa-list fa-2x" aria-hidden="true" {}
        }
    }

    async fn content(
        &self, user: &AuthenticatedUser<NonMutating>, permissions: &PermissionsManager, connection: &mut PgConnection,
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
                                (tr("claimed-player")) ": "
                            }
                            @match player_claim {
                                Some(ref claim) => {
                                    (P(&claim.player, Some("claimed-player"), &List::RatedPlus))
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
                                        (tr("claimed-player.verified"))
                                    }
                                    span.arrow.hover {}
                                },
                                Some(_) => i{(tr("claimed-player.unverified"))},
                                _ => {}
                            }
                        }
                    }
                    @if let Some(ref claim) = player_claim {
                        @if claim.verified {
                            div.overlined.pad.js-collapse-content #claims-claim-panel style="display:none" {
                                // It'd be neat to eliminate this feature and instead have this tied to the presence of a Box<dyn GeolocationProvider> state, but
                                // plumbing that information all the way here is... hella complicated.
                                @if cfg!(feature = "geolocation") {
                                    p.info-red.output style = "margin: 10px 0" {}
                                    p.info-green.output style = "margin: 10px 0" {}
                                    div.flex.no-stretch style="justify-content: space-between; align-items: center" {
                                        b {
                                            (tr("claim-geolocate"))
                                        }
                                        a.button.blue.hover #claims-geolocate-nationality {
                                            (tr("claim-geolocate.submit"))
                                        }
                                    }
                                    p {
                                        (tr("claim-geolocate.info"))
                                    }
                                }
                                div.cb-container.flex.no-stretch style="justify-content: space-between; align-items: center" {
                                    b {
                                        (tr("claim-lock-submissions"))
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
                                    (tr("claim-lock-submissions.info"))
                                }
                            }
                        }
                    }
                }
                @if let Some(claim) = player_claim {
                    @if claim.verified {
                        div.panel.fade {
                            h2.pad.underlined {
                                (tr("claim-records"))
                            }
                            p {
                                (PreEscaped(trp!(
                                    "claim-records.info",
                                    "record-approved-styled" = html! {
                                        span.ok { (tr("record-approved")) }
                                    }.into_string(),
                                    "record-submitted-styled" = html! {
                                        span.warn { (tr("record-submitted")) }
                                    }.into_string(),
                                    "record-rejected-styled" = html! {
                                        span.err { (tr("record-rejected")) }
                                    }.into_string(),
                                    "record-underconsideration-styled" = html! {
                                        span.consider { (tr("record-underconsideration")) }
                                    }.into_string()
                                )))
                            }
                            (paginator("claims-record-pagination", "/api/v1/records/"))
                        }
                    }
                }
                @if is_moderator {
                    div.panel.fade {
                        h2.pad.underlined {
                            (tr("claim-manager"))
                        }
                        p {
                            (tr("claim-manager.info-a"))
                            br;
                            (tr("claim-manager.info-b"))
                            br;
                            (tr("claim-manager.info-c"))
                            br;
                            (tr("claim-manager.info-d"))
                        }
                        (filtered_paginator("claim-pagination", "/api/v1/players/claims/"))
                    }
                }
            }
            div.right {
                div.panel.fade style = "display: none;"{
                    h2.underlined.pad {
                        (tr("claim-initiate-panel"))
                    }
                    p {
                        (tr("claim-initiate-panel.info"))
                    }
                    (filtered_paginator("claims-initiate-claim-pagination", "/api/v1/players/"))
                }
                div.panel.fade {
                    h2.underlined.pad {
                        (tr("claim-info-panel"))
                    }
                    p {
                        (tr("claim-info-panel.info-a"))
                        br;
                        (trp_html!(
                            "claim-info-panel.info-b",
                            "discord" = html! {
                                a.link href = (&self.0) { (tr("claim-info-panel.info-discord")) }
                            }
                        ))
                        br;
                        (tr("claim-info-panel.info-c"))
                    }
                }
                @if is_moderator {
                    div.panel.fade {
                        h2.underlined.pad {
                            (tr("claim-video-panel"))
                        }
                        p {
                            (tr("claim-video-panel.info"))
                        }
                        iframe."ratio-16-9"#claim-video style="width:100%;" allowfullscreen="" {}
                    }
                }
            }
        }
    }
}
