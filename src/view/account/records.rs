use crate::view::{demonlist::OverviewDemon, dropdown, paginator};
use maud::{html, Markup};

fn record_editor() -> Markup {
    html! {
        div.panel.fade.closable#edit style = "display: none" {
            span.plus.cross.hover {}
            h2.underlined.pad {
                "Edit Record #"
                span#edit-record-id {}
                " - "
            }
            form.flex.col#edit-record-form novalidate = "" {
                p.info-red.output {}
                span.form-input#edit-record-demon {
                    label for = "record_demon" {"ID of the demon the record is on:"}
                    input type = "number" name = "record_demon" value = "";
                    p.error {}
                }
                span.form-input#edit-record-player {
                    label for = "record_player" {"ID of the player of the record:"}
                    input type = "number" name = "record_player" value = "";
                    p.error {}
                }
                span.form-input#edit-record-progress {
                    label for = "record_progress" {"Progress made in the record:"}
                    input type = "number" name = "record_progress" min = "0" max = "100" value = "";
                    p.error {}
                }
                span.form-input#edit-record-video {
                    label for = "record_video" {"Video proof of legitimacy"}
                    input type = "url" name = "record_video";
                    p.error {}
                }
                p {
                    "All modifications to the record will only be saved upon clicking the button below. Selecting a different record, or leaving this page, will discard all unsaved modifications"
                }
                input.button.blue.hover type = "submit" style = "margin: 15px auto 0px;" value="Save edit(s)";
            }
        }
    }
}

fn record_manager(demons: &[OverviewDemon]) -> Markup {
    html! {
        div.panel.fade#record-manager {
            h2.underlined.pad {
                "Record Manager - "
                (dropdown("All", html! {
                    li.white.hover.underlined data-value = "All Demons"
                     {"All Demons"}
                }, demons.into_iter().map(|demon| html!(li.white.hover data-value = (demon.id) data-display = (demon.name) {b{"#"(demon.position) " - " (demon.name)} br; {"by "(demon.publisher)}}))))
            }
            div.flex.viewer {
                (paginator("record-pagination", "/api/v1/records/"))
                p.viewer-welcome {
                    "Click on a record on the left to get started!"
                }
                div.viewer-content {
                    div {
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
    }
}

fn manager_help() -> Markup {
    html! {
        div.panel.fade {
            h1.underlined.pad {
                "Manage Records"
            }
            p {
                "Use the list on the left to select records for editing/viewing. Use the panel on the right to filter the record list by status, player, etc.. Clicking the 'All Demons' field at the top allows to filter by demon."
            }
            p {
                "There are four possible record states a record can be in: " i { "'rejected', 'approved', 'submitted'" } " and " i { "'under consideration'" } ". For simplicity of explanation we will assume that 'Bob' is a player and 'Cataclysm' is a demon he has a record on."
                ul {
                    li {
                        b{"Rejected: "} "If the record is 'rejected', it means that Bob has no other record in other states on Cataclysm and no submissions for Bob on Cataclysm are possible. Conversely, this means if Bob has a record on Catalysm that's not rejected, we immediately know that no rejected record for Bob on Cataclysm exists. "
                        br;
                        "Rejecting any record of Bob's on Cataclysm will delete all other record's of Bob on Cataclysm to ensure the above uniqueness"
                    }
                    li {
                        b{"Approved: "} "If the record is 'approved', it means that no submissions with less progress than the 'approved' record exist or are permitted."
                        br;
                        "Changing a record to 'approved' will delete all submissions for Bob on Cataclysm with less progress"
                    }
                    li {
                        b {"Submitted: "} "If the record is 'submitted', no further constraints on uniqueness are in place. This means that multiple submissions for Bob on Cataclysm are possible, as long as they provide different video links. However, due to the above, all duplicates are deleted as soon as one of the submissions is accepted or rejected"
                    }
                    li {
                        b {"Under Consideration: "} "If the record is 'under consideration' it is conceptually still a submission. The only difference is, that no more submissions for Bob on Cataclysm are allowed now."
                    }
                }
            }
            p {
                b { "Note: " }
                "If a player is banned, they cannot have accepted/submitted records on the list. All records marked as 'submitted' are deleted, all others are changed to 'rejected'"
            }
            p {
                b { "Note: " }
                "Banning a submitter will delete all their submissions that still have the status 'Submitted'. Records submitted by them that were already accepted/rejected will not be affected"
            }
        }
    }
}

fn status_selector() -> Markup {
    // FIXME: no vec
    let dropdown_items = vec![
        html! {
            li.white.hover data-value = "approved" {"Approved"}
        },
        html! {
            li.white.hover data-value = "submitted" {"Submitted"}
        },
        html! {
            li.white.hover data-value = "rejected" {"Rejected"}
        },
        html! {
            li.white.hover data-value = "under consideration" {"Under Consideration"}
        },
    ];

    html! {
        div.panel.fade#status-filter-panel style = "overflow: visible" {
            h2.underlined.pad {
                "Filter"
            }
            p {
                "Filter by record status"
            }
            (dropdown("All", html! {
                li.white.hover.underlined data-value = "All" {"All"}
            }, dropdown_items.into_iter()))
        }
    }
}

fn player_selector() -> Markup {
    html! {
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
                input.button.blue.hover type = "submit" style = "margin: 15px auto 0px;" value="Find by ID";
            }
            form.flex.col#record-filter-by-player-name-form novalidate = "" {
                p.info-red.output {}
                span.form-input#record-player-name {
                    label for = "name" {"Player name:"}
                    input required = "" type = "text" name = "name";
                    p.error {}
                }
                input.button.blue.hover type = "submit" style = "margin: 15px auto 0px;" value="Find by name";
            }
        }
    }
}

pub(super) fn page(demons: &[OverviewDemon]) -> Markup {
    html! {
        div.m-center.flex.tab-content.container data-tab-id = "3" {
            div.left {
                (record_manager(demons))
                (manager_help())
            }
            div.right {
                (status_selector())
                (player_selector())
            }
        }
    }
}
