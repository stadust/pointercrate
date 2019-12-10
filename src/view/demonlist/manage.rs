use crate::view::{account::AccountPage, dropdown, paginator};
use maud::{html, Markup};

impl AccountPage {
    pub(in crate::view) fn record_page(&self) -> Markup {
        html! {
            div.m-center.flex.tab-content.container data-tab-id = "3" {
                div.left {
                    div.panel.fade#record-manager {
                        h2.underlined.pad {
                            "Record Manager (Alpha)"
                        }
                        div.flex {
                            (paginator("record-pagination", "/records/"))
                            div {
                                p.viewer-welcome style = "text-align: center" {
                                    "Click on a record on the left to get started!"
                                }
                                div.viewer-content style = "display:none" {
                                    div.flex.col {
                                        h3 style = "font-size:1.4em; overflow: hidden" { "Record #" i#record-id{}}

                                        iframe."ratio-16-9"#record-video style="width:90%; margin: 15px 5%" allowfullscreen="" {"Verification Video"}
                                        div.stats-container.flex.space  {
                                            span{
                                                b {
                                                    "Video Link:"
                                                }
                                                br;
                                                a.link#record-video-link target = "_blank" {}
                                            }
                                        }
                                        div.stats-container.flex.space {
                                            span {
                                                b {
                                                    "Demon:"
                                                }
                                                br;
                                                span#record-demon {}
                                            }
                                            span {
                                                b {
                                                    "Record Holder:"
                                                }
                                                br;
                                                span#record-holder {}
                                            }
                                            span {
                                                b {
                                                    "Record status:"
                                                }
                                                br;
                                                span#record-status {}
                                            }
                                        }
                                        div.stats-container.flex.space {
                                            span {
                                                b {
                                                    "Progress:"
                                                }
                                                br;
                                                span#record-progress {}
                                            }
                                            span {
                                                b {
                                                    "Submitter ID:"
                                                }
                                                br;
                                                span#record-submitter {}
                                            }
                                        }

                                        div.stats-container.flex.space {
                                            span {
                                                b {
                                                    "Notes:"
                                                }
                                                br;
                                                span#record-notes {}
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    div.panel.fade {
                        h1.underlined.pad {
                            "Manage Record"
                        }
                        p {
                            "Use the list on the left to select records for editing/viewing. Use the panel on the right to filter the record list by submission, player, etc.. Clicking the 'All Demons' field at the top allows to filter by demon."
                        }
                        p {
                            b { "Note: " }
                            "If a player is banned, they cannot have accepted/submitted records on the list, and all records they potentially once had are marked as rejected"
                        }
                        p {
                            b { "Note: " }
                            "Banning a submitter will delete all their submissions that still have the status 'Submitted'. Records submitted by them that were already accepted/rejected will not be affected"
                        }
                    }

                }
                div.right {
                    div.panel.fade#status-filter-panel style = "overflow: visible" {
                        h2.underlined.pad {
                            "Filter"
                        }
                        p {
                            "Filter by record status"
                        }
                        (dropdown("All", html! {
                            li.white.hover.underlined data-value = "All" {"All"}
                        }, ["Approved", "Submitted", "Rejected"].into_iter().map(|s| html!(li.white.hover data-value = (s) {(s)}))))
                    }
                    div.panel.fade {
                        h2.underlined.pad {
                            "Filter by player"
                        }
                        p {
                            "Players can be uniquely identified by name and ID. Entering either in the appropriate place below will filter the view on the left. Right now the only way to reset this filter is to reload the page. Sorry!"
                        }
                        form.flex.col.underlined.pad#record-filter-by-player-id-form novalidate = "" {
                            p.info-red.output {}
                            span.form-input#record-player-id {
                                label for = "id" {"Player ID:"}
                                input required = "" type = "number" name = "id" min = "0" style="width:93%"; // FIXME: I have no clue why the input thinks it's a special snowflake and fucks up its width, but I dont have the time to fix it
                                p.error {}
                            }
                            input.button.blue.hover.slightly-round type = "submit" style = "margin: 15px auto 0px;" value="Find by ID";
                        }
                        form.flex.col#record-filter-by-player-name-form novalidate = "" {
                            p.info-red.output {}
                            span.form-input#record-player-name {
                                label for = "name" {"Player name:"}
                                input required = "" type = "text" name = "name";
                                p.error {}
                            }
                            input.button.blue.hover.slightly-round type = "submit" style = "margin: 15px auto 0px;" value="Find by name";
                        }
                    }
                }
            }
        }
    }
}
