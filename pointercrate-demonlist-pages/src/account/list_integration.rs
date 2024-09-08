use log::error;
use maud::{html, Markup, PreEscaped};
use pointercrate_core::{error::PointercrateError, permission::PermissionsManager};
use pointercrate_core_pages::{
    error::ErrorFragment,
    util::{filtered_paginator, paginator},
};
use pointercrate_demonlist::player::claim::PlayerClaim;
use pointercrate_user::{auth::AuthenticatedUser, MODERATOR};
use pointercrate_user_pages::account::AccountPageTab;
use sqlx::PgConnection;

pub struct ListIntegrationTab(#[doc = "discord invite url"] pub &'static str);

#[async_trait::async_trait]
impl AccountPageTab for ListIntegrationTab {
    fn should_display_for(&self, _permissions_we_have: u16, _permissions: &PermissionsManager) -> bool {
        true
    }

    fn initialization_script(&self) -> String {
        "/static/demonlist/js/account/integration.js?v=4".into()
    }

    fn tab_id(&self) -> u8 {
        7
    }

    fn tab(&self) -> Markup {
        html! {
            b {
                "List Integration"
            }
            (PreEscaped("&nbsp;&nbsp;"))
            i class = "fa fa-list fa-2x" aria-hidden="true" {}
        }
    }

    async fn content(&self, user: &AuthenticatedUser, permissions: &PermissionsManager, connection: &mut PgConnection) -> Markup {
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
                                "Claimed Player: "
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
                                        "Verified"
                                    }
                                    span.arrow.hover {}
                                },
                                Some(_) => i{"Unverified"},
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
                                        "Geolocate statsviewer flag:"
                                    }
                                    a.button.blue.hover #claims-geolocate-nationality {
                                        "Go"
                                    }
                                }
                                p {
                                    "Clicking the above button let's you set your claimed player's statsviewer flag via IP Geolocation. To offer this functionality, pointercrate uses "
                                    a.link href = "https://www.abstractapi.com/ip-geolocation-api" { "abstract's IP geolocation API"}
                                    ". Clicking the above button also counts as your consent for pointercrate to send your IP to abstract."
                                }
                                div.cb-container.flex.no-stretch style="justify-content: space-between; align-items: center" {
                                    b {
                                        "Lock Submissions:"
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
                                    "Whether submissions for your claimed player should be locked, meaning only you will be able to submit records for your claimed player (and only while logged in to this account holding the verified claim)"
                                }
                            }
                        }
                    }
                }
                @if let Some(claim) = player_claim {
                    @if claim.verified {
                        div.panel.fade {
                            h2.pad.underlined {
                                "Your claimed player's records"
                            }
                            p {
                                "A list of your claimed player's records, including all under consideration and rejected records and all submissions. Use this to track the status of your submissions. Clicking on a record will pull up any public notes a list mod left on the given record. The background color of each record tells you whether the record is "
                                span  style = "background-color: #E9FAE3" { "Approved"  } ", "
                                span style = "background-color: #F7F7E0" { "Unchecked" } ", "
                                span style = "background-color: #F8DCE4" { "Rejected" } " or "
                                span style = "background-color: #D8EFF3" { "Under Consideration" } "."
                            }
                            (paginator("claims-record-pagination", "/api/v1/records/"))
                        }
                    }
                }
                @if is_moderator {
                    div.panel.fade {
                        h2.pad.underlined {
                            "Manage Claims"
                        }
                        p {
                            "Manage claims using the interface below. The list can be filtered by player and user using the panels on the right. Invalid claims should be deleted using the trash icon. "
                            br;
                            "To verify a claim, click the checkmark. Only verify claims you have verified to be correct (this will probably mean talking to the player that's being claimed, and asking if they initiated the claim themselves, or if the claim is malicious)."
                            br;
                            "Once a claim on a player is verified, all other unverified claims on that player are auto-deleted. Users cannot put new, unverified claims on players that have a verified claim on them."
                            br;
                            "A claim with a green background is verified, a claim with a blue background is unverified/unchecked"
                        }
                        (filtered_paginator("claim-pagination", "/api/v1/players/claims/"))
                    }
                }
            }
            div.right {
                div.panel.fade style = "display: none;"{
                    h2.underlined.pad {
                        "Initiate Claim"
                    }
                    p {
                        "Select the player you wish to claim below"
                    }
                    (filtered_paginator("claims-initiate-claim-pagination", "/api/v1/players/"))
                }
                div.panel.fade {
                    h2.underlined.pad {
                        "Claiming 101"
                    }
                    p {
                        "Player claiming is the process of associated a demonlist player with a pointercrate user account. A verified claim allows you to to modify some of the player's properties, such as nationality. "
                        br;
                        "To initiate a claim, click the pen left of the 'Claimed Player' heading. Once initiated, you have an unverified claim on a player. These claims will then be manually verified by members of the pointercrate team. You can request verification in " a.link href=(self.0) {"this discord server"} "."
                        br;
                        "You cannot initiate a claim on a player that already has a verified claim by a different user on it. "
                    }
                }
                @if is_moderator {
                    div.panel.fade {
                        h2.underlined.pad {
                            "Record video"
                        }
                        p {
                            "Clicking a claim in the 'Manage Claims' panel will pull up a random video of an approved record by the claimed player."
                        }
                        iframe."ratio-16-9"#claim-video style="width:100%;" allowfullscreen="" {}
                    }
                }
            }
        }
    }
}
