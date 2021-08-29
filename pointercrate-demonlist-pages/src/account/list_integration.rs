use maud::{html, Markup, PreEscaped};
use pointercrate_core::{error::PointercrateError, permission::PermissionsManager};
use pointercrate_core_pages::{error::ErrorFragment, PageFragment, Script};
use pointercrate_demonlist::player::claim::PlayerClaim;
use pointercrate_user::{sqlx::PgConnection, User};
use pointercrate_user_pages::account::AccountPageTab;

pub struct ListIntegrationTab;

#[async_trait::async_trait]
impl AccountPageTab for ListIntegrationTab {
    fn should_display_for(&self, _user: &User, _permissions: &PermissionsManager) -> bool {
        true
    }

    fn additional_scripts(&self) -> Vec<Script> {
        vec![]
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

    async fn content(&self, user: &User, permissions: &PermissionsManager, connection: &mut PgConnection) -> Markup {
        let player_claim = match PlayerClaim::by_user(user.id, connection).await {
            Ok(player_claim) => player_claim,
            Err(err) =>
                return ErrorFragment {
                    status: err.status_code(),
                    reason: "Internal Server Error".to_string(),
                    message: err.to_string(),
                }
                .body_fragment(),
        };

        html! {
            div.left {
                div.panel.fade.flex.no-stretch style="padding: 5px; text-align: left; justify-content: space-between" {
                    @match player_claim {
                        Some(claim) => {
                            div {
                                b {
                                    "Claim on player: "
                                }
                                i {
                                    (claim.player_id)
                                }
                                br;
                                b {
                                    "Status: "
                                }
                                i {
                                    @if claim.verified {
                                        "verified"
                                    }
                                    @else {
                                        "unverified"
                                    }
                                }
                            }
                            a.button.blue.hover {
                                "Change claim"
                            }
                        },
                        None => {
                            i {
                                "No player currently claimed"
                            }
                            a.button.blue.hover {
                                "Initiate claim"
                            }
                        }
                    }
                }
                div style="height: 50px" {} // to make sure that the footer doesnt float. if it floats, the user page is the only one without a scrollbar at the right, which causes jumpyness when switching tabs.
            }
            div.right {
                div.panel.fade {

                }
            }
        }
    }
}
