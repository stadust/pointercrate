use crate::view::{demonlist::OverviewDemon, dropdown, paginator};
use maud::{html, Markup};

fn record_editor() -> Markup {
    html! {
        div.panel.fade.closable#edit-record style = "display: none" {
            span.plus.cross.hover {}
            h2.underlined.pad {
                "Edit Record #"
                span#edit-record-id {}
            }
            form.flex.col#edit-record-form novalidate = "" {
                p.info-red.output {}
                p.info-green.output {}
                span.form-input#edit-record-demon-id {
                    label for = "record_demon" {"ID of the demon the record should be on:"}
                    input type = "number" name = "demon_id" value = "";
                    p.error {}
                }
                span.form-input#edit-record-demon-name {
                    label for = "record_demon" {"Name of the demon the record should be on:"}
                    input type = "text" name = "demon" value = "";
                    p.error {}
                }
                span.form-input#edit-record-player {
                    label for = "record_player" {"Name of the player of the record:"}
                    input type = "text" name = "player" value = "";
                    p.error {}
                }
                span.form-input#edit-record-progress {
                    label for = "record_progress" {"Progress made in the record:"}
                    input type = "number" name = "progress" min = "0" max = "100" value = "";
                    p.error {}
                }
                span.form-input#edit-record-video {
                    label for = "record_video" {"Video proof of legitimacy:"}
                    input type = "url" name = "video";
                    p.error {}
                }
                p{
                    b {"Important: "}
                    "Not all fields have to be filled out! Leaving a field empty leaves that value unchanged! The fields 'demon id' and 'demon name' are mutually exclusive"
                }
                p {
                    "All modifications to the record will only be saved upon clicking the button below. Selecting a different record, or leaving the staff area, will discard all unsaved modifications. Navigating to a different tab above is fine. Selecting a different record below instead targets the newly selected record for modification!"
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
                    li.white.hover.underlined data-value = "All"
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
                            h3 style = "font-size:1.1em; margin-top: 10px" {
                                "Record #"
                                i#record-id {}
                                " - "
                                div.dropdown-menu.js-search#edit-record-status style = "max-width: 220px"{
                                    input type="text" style = "color: #444446; font-weight: bold;";
                                    div.menu {
                                        ul {
                                            li.white.hover data-value="approved" {"Approved"}
                                            li.white.hover data-value="rejected" {"Rejected"}
                                            li.white.hover data-value="under consideration" {"Under Consideration"}
                                            li.white.hover data-value="submitted" {"Submitted"}
                                        }
                                    }
                                }
                            }

                            iframe."ratio-16-9"#record-video style="width:90%; margin: 15px 5%" allowfullscreen="" {"Verification Video"}
                            p.info-red.output style = "margin: 10px" {}
                            p.info-green.output style = "margin: 10px" {}
                            div.stats-container.flex.space  {
                                span{
                                    b {
                                        i.fa.fa-pencil.clickable#record-video-pen aria-hidden = "true" {} " Video Link:"
                                    }
                                    br;
                                    a.link#record-video-link target = "_blank" {}
                                }
                            }
                            div.stats-container.flex.space {
                                span {
                                    b {
                                        i.fa.fa-pencil.clickable#record-demon-pen aria-hidden = "true" {} " Demon:"
                                    }
                                    br;
                                    span#record-demon {}
                                }
                                span {
                                    b {
                                        i.fa.fa-pencil.clickable#record-player-pen aria-hidden = "true" {} " Record Holder:"
                                    }
                                    br;
                                    span#record-holder {}
                                }
                            }
                            div.stats-container.flex.space {
                                span {
                                    b {
                                        i.fa.fa-pencil.clickable#record-progress-pen aria-hidden = "true" {} " Progress:"
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
                            div.flex.no-stretch style = "margin: 15px 10px 0px; justify-content: space-evenly" {
                                span.button.blue.hover.js-scroll data-destination = "edit-record" data-reveal = "true" {"Edit Record"};
                                span.button.red.hover#record-delete {"Delete Record"};
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

fn note_adder() -> Markup {
    html! {
        div.panel.fade.closable#add-record-note style = "display: none" {
            span.plus.cross.hover {}
            div.button.blue.hover.small style = "width: 100px; margin-bottom: 10px"{
                "Add"
            }
            p.info-red.output {}
            textarea style = "width: 100%" placeholder = "Add note here. Click 'Add' above when done!"{}
        }
    }
}

pub(super) fn page(demons: &[OverviewDemon]) -> Markup {
    html! {
        div.m-center.flex.tab-content.container data-tab-id = "3" {
            div.left {
                (crate::view::demonlist::submission_panel())
                (record_editor())
                (record_manager(demons))
                (note_adder())
                div.panel.fade#record-notes-container style = "display:none" {
                    div.white.hover.clickable#add-record-note-open {
                        b {"Add Note"}
                    }
                    div#record-notes {} // populated by javascript when a record is clicked
                }
                (manager_help())
            }
            div.right {
                (status_selector())
                (player_selector())
                (crate::view::demonlist::submit_panel())
            }
        }
    }
}
