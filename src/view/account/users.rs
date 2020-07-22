use crate::view::filtered_paginator;
use maud::{html, Markup};

pub(super) fn page(is_admin: bool) -> Markup {
    html! {
        div.m-center.flex.tab-content.container data-tab-id = "2" {
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
                                    @if is_admin {
                                        label.cb-container.form-input#perm-extended for = "extended"  {
                                            i{"Extended access"}
                                            input type = "checkbox" name = "extended";
                                            span.checkmark {}
                                        }
                                    }
                                    @else {
                                        label.cb-container.form-input#perm-extended for = "extended" style = "opacity: .3" {
                                            i{"Extended access"}
                                            input type = "checkbox" name = "extended" disabled = "";
                                            span.checkmark {}
                                        }
                                    }
                                    label.form-input.cb-container#perm-list-helper for = "helper" {
                                        i {"List Helper"}
                                        input type = "checkbox" name = "helper";
                                        span.checkmark {}
                                    }
                                    label.form-input.cb-container#perm-list-mod for = "mod" {
                                        i {"List Moderator"}
                                        input type = "checkbox" name = "mod";
                                        span.checkmark {}
                                    }
                                    @if is_admin {
                                        label.form-input.cb-container#perm-list-admin for = "admin" {
                                            i {"List Administrator"}
                                            input type = "checkbox" name = "admin";
                                            span.checkmark {}
                                        }
                                    }
                                    @else {
                                        label.form-input.cb-container#perm-list-admin for = "admin" style = "opacity: .3"{
                                            i {"List Administrator"}
                                            input type = "checkbox" name = "admin" disabled = "";
                                            span.checkmark {}
                                        }
                                    }
                                    @if is_admin {
                                        label.form-input.cb-container#perm-mod for = "mod2" {
                                            i {"Moderator"}
                                            input type = "checkbox" name = "mod2";
                                            span.checkmark {}
                                        }
                                    }
                                    @else {
                                        label.form-input.cb-container#perm-mod for = "mod2" style = "opacity: .3"{
                                            i {"Moderator"}
                                            input type = "checkbox" name = "mod2" disabled = "";
                                            span.checkmark {}
                                        }
                                    }

                                    @if is_admin {
                                        label.form-input.cb-container#perm-admin for = "admin2" {
                                            i {"Administrator"}
                                            input type = "checkbox" name = "admin2";
                                            span.checkmark {}
                                        }
                                    }
                                    @else {
                                        label.form-input.cb-container#perm-admin for = "admin2" style = "opacity: .3"{
                                            i {"Administrator"}
                                            input type = "checkbox" name = "admin2" disabled = "";
                                            span.checkmark {}
                                        }
                                    }
                                }
                                div.flex.no-stretch {
                                    @if is_admin {
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
