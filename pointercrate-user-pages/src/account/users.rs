use crate::account::AccountPageTab;
use maud::{html, Markup, PreEscaped};
use pointercrate_core::{permission::PermissionsManager, view::misc::filtered_paginator};
use pointercrate_user::{sqlx::PgConnection, User, ADMINISTRATOR, MODERATOR};

pub struct UsersTab;

#[async_trait::async_trait]
impl AccountPageTab for UsersTab {
    fn should_display_for(&self, user: &User, permissions: &PermissionsManager) -> bool {
        permissions.require_permission(user.permissions, MODERATOR).is_ok()
    }

    fn additional_scripts(&self) -> Vec<String> {
        vec!["/static/js/account/users.js".to_string()]
    }

    fn tab(&self) -> Markup {
        html! {
            b {
                "Users"
            }
            (PreEscaped("&nbsp;&nbsp;"))
            i class = "fa fa-users fa-2x" aria-hidden="true" {}
        }
    }

    async fn page(&self, user: &User, permissions: &PermissionsManager, connection: &mut PgConnection) -> Markup {
        let assignable_permissions = permissions.assignable_by_bits(user.permissions);

        html! {
            div.left {
                div.panel.fade {
                    h2.underlined.pad {
                        "Pointercrate Account Manager"
                    }

                    div.flex.viewer {
                        (filtered_paginator("user-pagination", "/api/v1/users/"))
                        p.viewer-welcome {
                            "Click on a user on the left to get started!"
                        }
                        div.viewer-content {
                            div.stats-container.flex.space {
                                span {
                                    b {
                                        "Username:"
                                    }
                                    br;
                                    span#user-user-name {}
                                }
                                span {
                                    b {
                                        "Display Name:"
                                    }
                                    br;
                                    span#user-display-name {}
                                }
                                span {
                                    b {
                                        "User ID:"
                                    }
                                    br;
                                    span#user-user-id {}
                                }
                            }
                            form.flex.col.pad#patch-permissions novalidate = "" style="display:none" {
                                p.info-red.output {}
                                p.info-green.output {}

                                div.stats-container.flex.space.col style = "align-items: center" {
                                    b {
                                        "Permissions:"
                                    }
                                    @for permission in assignable_permissions {
                                        @let name_in_snake_case = permission.name().to_lowercase().replace(" ", "-");

                                        label.cb-container.form-input for = (name_in_snake_case) {
                                            i {
                                                (permission.name())
                                            }
                                            input type = "checkbox" name = (name_in_snake_case) bit = (permission.bit());
                                            span.checkmark {}
                                        }
                                    }
                                }
                                div.flex.no-stretch {
                                    @if user.has_permission(ADMINISTRATOR) {
                                        input.button.red.hover#delete-user type = "button" style = "margin: 15px auto 0px;" value="Delete user";
                                    }
                                    input.button.blue.hover type = "submit" style = "margin: 15px auto 0px;" value="Edit user";
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
                        "Find users"
                    }
                    p {
                        "Users can be uniquely identified by name and ID. To modify a user's account, you need their ID. If you know neither, try looking in the list below"
                    }
                    form.flex.col.pad#find-id-form novalidate = "" {
                        p.info-red.output {}
                        span.form-input#find-id {
                            label for = "id" {"User ID:"}
                            input required = "" type = "number" name = "id" min = "0" style="width:93%"; // FIXME: I have no clue why the input thinks it's a special snowflake and fucks up its width, but I dont have the time to fix it
                            p.error {}
                        }
                        input.button.blue.hover type = "submit" style = "margin: 15px auto 0px;" value="Find by ID";
                    }
                }
            }
        }
    }
}
