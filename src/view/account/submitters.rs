use crate::view::paginator;
use maud::{html, Markup};

pub(super) fn page() -> Markup {
    html! {
        div.m-center.flex.tab-content.container data-tab-id = "6" {
            div.left {
                div.panel.fade {
                    h2.underlined.pad {
                        "Submitter Manager"
                    }
                    div.flex.viewer {
                        (paginator("submitter-pagination", "/api/v1/submitters/"))
                        p.viewer-welcome {
                            "Click on a submitter on the left to get started!"
                        }
                        div.viewer-content {
                            div.flex.col{
                                h3 style = "font-size:1.1em; margin: 10px 0" {
                                    "Submitter #"
                                    i#submitter-submitter-id {}
                                }
                                p {
                                    "Welcome to the submitter manager. Here you can ban or unban submitters with an absolute revolutionary UI that totally isn't a stright up copy of the player UI, just with even more emptiness. "
                                }
                                p {
                                    "Banning a submitter will delete all records they have submitted and which are still in the 'submitted' state. All submissions of their which are approved, rejected or under consideration are untouched. "
                                }
                                p.info-red.output style = "margin: 10px" {}
                                p.info-green.output style = "margin: 10px" {}
                                div.stats-container.flex.space {
                                    span {
                                        b {
                                            "Banned:"
                                        }
                                        br;
                                        div.dropdown-menu.js-search#edit-submitter-banned style = "max-width: 50px"{
                                            input type="text" style = "color: #444446; font-weight: bold;";
                                            div.menu {
                                                ul {
                                                    li.white.hover data-value="true" {"yes"}
                                                    li.white.hover data-value="false" {"no"}
                                                }
                                            }
                                        }
                                    }
                                }
                                span.button.blue.hover#submitter-list-records style = "margin: 15px auto 0px" {"Show records in record manager"};
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
                "Search submitter by ID"
            }
            p {
                "Submitters can be uniquely identified by ID. Entering a submitters's ID below will select it on the left (provided the submitter exists)"
            }
            form.flex.col#submitter-search-by-id-form novalidate = "" {
                p.info-red.output {}
                span.form-input#search-submitter-id {
                    label for = "id" {"Submitter ID:"}
                    input required = "" type = "number" name = "id" min = "0" style="width:93%";
                    p.error {}
                }
                input.button.blue.hover type = "submit" style = "margin: 15px auto 0px;" value="Find by ID";
            }
        }
    }
}
