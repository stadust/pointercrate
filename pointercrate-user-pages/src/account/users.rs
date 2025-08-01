use crate::account::AccountPageTab;
use maud::{html, Markup, PreEscaped};
use pointercrate_core::{
    localization::tr,
    permission::{Permission, PermissionsManager},
};
use pointercrate_core_pages::util::filtered_paginator;
use pointercrate_user::{
    auth::{AuthenticatedUser, NonMutating},
    ADMINISTRATOR,
};
use sqlx::PgConnection;

pub struct UsersTab(pub Vec<Permission>);

#[async_trait::async_trait]
impl AccountPageTab for UsersTab {
    fn should_display_for(&self, permissions_we_have: u16, permissions: &PermissionsManager) -> bool {
        for perm in &self.0 {
            if permissions.require_permission(permissions_we_have, *perm).is_ok() {
                return true;
            }
        }

        false
    }

    fn initialization_script(&self) -> String {
        "/static/user/js/account/users.js".into()
    }

    fn tab_id(&self) -> u8 {
        2
    }

    fn tab(&self) -> Markup {
        html! {
            b {
                (tr("users"))
            }
            (PreEscaped("&nbsp;&nbsp;"))
            i class = "fa fa-users fa-2x" aria-hidden="true" {}
        }
    }

    async fn content(
        &self, user: &AuthenticatedUser<NonMutating>, permissions: &PermissionsManager, _connection: &mut PgConnection,
    ) -> Markup {
        let mut assignable_permissions = permissions
            .assignable_by_bits(user.user().permissions)
            .into_iter()
            .collect::<Vec<_>>();
        assignable_permissions.sort_by_key(|perm| perm.bit());

        html! {
            div.left {
                div.panel.fade {
                    h2.underlined.pad {
                        (tr("user-viewer"))
                    }

                    div.flex.viewer {
                        (filtered_paginator("user-pagination", "/api/v1/users/"))
                        p.viewer-welcome {
                            (tr("user-viewer.welcome"))
                        }
                        div.viewer-content {
                            div.stats-container.flex.space {
                                span {
                                    b {
                                        (tr("user-username"))
                                    }
                                    br;
                                    span #user-user-name {}
                                }
                                span {
                                    b {
                                        (tr("user-displayname"))
                                    }
                                    br;
                                    span #user-display-name {}
                                }
                                span {
                                    b {
                                        (tr("user-id"))
                                    }
                                    br;
                                    span #user-user-id {}
                                }
                            }
                            form.flex.col.pad #patch-permissions novalidate = "" style="display:none" {
                                p.info-red.output {}
                                p.info-green.output {}

                                @if !assignable_permissions.is_empty() {
                                    div.stats-container.flex.space.col style = "align-items: center" {
                                        b {
                                            (tr("user-permissions"))
                                        }
                                        @for permission in assignable_permissions {
                                            @let permission_name = tr(permission.text_id());

                                            label.cb-container.form-input #(permission.text_id()) for = (permission.text_id()) data-bit = (permission.bit()) {
                                                i {
                                                    (permission_name)
                                                }
                                                input type = "checkbox" name = (permission.text_id());
                                                span.checkmark {}
                                            }
                                        }
                                    }
                                }
                                div.flex.no-stretch {
                                    @if user.user().has_permission(ADMINISTRATOR) {
                                        input.button.red.hover #delete-user type = "button" style = "margin: 15px auto 0px;" value=(tr("user-viewer.delete-user"));
                                    }
                                    input.button.blue.hover type = "submit" style = "margin: 15px auto 0px;" value=(tr("user-viewer.edit-user"));
                                }
                            }
                        }
                    }
                }
                div style="height: 50px" {} // to make sure that the footer doesnt float. if it floats, the user page is the only one without a scrollbar at the right, which causes jumpyness when switching tabs.
            }
            div.right {
                div.panel.fade {
                    h2.underlined.pad {
                        (tr("user-idsearch-panel"))
                    }
                    p {
                        (tr("user-idsearch-panel.info"))
                    }
                    form.flex.col.pad #find-id-form novalidate = "" {
                        p.info-red.output {}
                        span.form-input #find-id {
                            label for = "id" {(tr("user-idsearch-panel.id-field")) }
                            input required = "" type = "number" name = "id" min = "0" style="width:93%"; // FIXME: I have no clue why the input thinks it's a special snowflake and fucks up its width, but I dont have the time to fix it
                            p.error {}
                        }
                        input.button.blue.hover type = "submit" style = "margin: 15px auto 0px;" value=(tr("user-idsearch-panel.submit"));
                    }
                }
            }
        }
    }
}
