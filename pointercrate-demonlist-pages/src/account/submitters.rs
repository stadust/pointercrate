use maud::{html, Markup, PreEscaped};
use pointercrate_core::{localization::tr, permission::PermissionsManager};
use pointercrate_core_pages::util::paginator;
use pointercrate_demonlist::LIST_MODERATOR;
use pointercrate_user::auth::{AuthenticatedUser, NonMutating};
use pointercrate_user_pages::account::AccountPageTab;
use sqlx::PgConnection;

pub struct SubmittersPage;

#[async_trait::async_trait]
impl AccountPageTab for SubmittersPage {
    fn should_display_for(&self, permissions_we_have: u16, permissions: &PermissionsManager) -> bool {
        permissions.require_permission(permissions_we_have, LIST_MODERATOR).is_ok()
    }

    fn initialization_script(&self) -> String {
        "/static/demonlist/js/account/submitter.js".into()
    }

    fn tab_id(&self) -> u8 {
        6
    }

    fn tab(&self) -> Markup {
        html! {
            b {
                (tr("submitters"))
            }
            (PreEscaped("&nbsp;&nbsp;"))
            i class = "fa fa-eye fa-2x" aria-hidden="true" {}
        }
    }

    async fn content(
        &self, _user: &AuthenticatedUser<NonMutating>, _permissions: &PermissionsManager, _connection: &mut PgConnection,
    ) -> Markup {
        html! {
            div.left {
                div.panel.fade {
                    h2.underlined.pad {
                        (tr("submitter-manager"))
                    }
                    div.flex.viewer {
                        (paginator("submitter-pagination", "/api/v1/submitters/"))
                        p.viewer-welcome {
                            (tr("submitter-viewer.welcome"))
                        }
                        div.viewer-content {
                            div.flex.col{
                                h3 style = "font-size:1.1em; margin: 10px 0" {
                                    (tr("submitter-viewer"))
                                    i #submitter-submitter-id {}
                                }
                                p {
                                    (tr("submitter-viewer.info-a"))
                                }
                                p {
                                    (tr("submitter-viewer.info-b"))
                                }
                                p.info-red.output style = "margin: 10px" {}
                                p.info-green.output style = "margin: 10px" {}
                                div.stats-container.flex.space {
                                    span {
                                        b {
                                            (tr("submitter-banned"))
                                        }
                                        br;
                                        div.dropdown-menu.js-search #edit-submitter-banned style = "max-width: 50px" {
                                            div{
                                                input type="text" style = "font-weight: bold;";
                                            }
                                            div.menu {
                                                ul {
                                                    li.white.hover data-value="true" {(tr("submitter-banned.yes"))}
                                                    li.white.hover data-value="false" {(tr("submitter-banned.no"))}
                                                }
                                            }
                                        }
                                    }
                                }
                                span.button.blue.hover #submitter-list-records style = "margin: 15px auto 0px" {(tr("submitter-viewer.records-redirect"))};
                            }
                        }
                    }
                }
                div style="height: 50px" {} // to make sure that the footer doesnt float. if it floats, the user page is the only one without a scrollbar at the right, which causes jumpyness when switching tabs.
            }
            div.right {
                (submitter_selector())
            }
        }
    }
}

fn submitter_selector() -> Markup {
    html! {
        div.panel.fade {
            h2.underlined.pad {
                (tr("submitter-idsearch-panel"))
            }
            p {
                (tr("submitter-idsearch-panel.info"))
            }
            form.flex.col #submitter-search-by-id-form novalidate = "" {
                p.info-red.output {}
                span.form-input #search-submitter-id {
                    label for = "id" {(tr("submitter-idsearch-panel.id-field")) }
                    input required = "" type = "number" name = "id" min = "0" style="width:93%";
                    p.error {}
                }
                input.button.blue.hover type = "submit" style = "margin: 15px auto 0px;" value=(tr("submitter-idsearch-panel.submit"));
            }
        }
    }
}
