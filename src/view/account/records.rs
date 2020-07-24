use crate::view::{demonlist::OverviewDemon, dropdown, paginator};
use maud::{html, Markup};

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

                        iframe."ratio-16-9"#record-video style="width:90%; margin: 15px 5%" allowfullscreen="" {"Video"}
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
                                    i.fa.fa-pencil.clickable#record-holder-pen aria-hidden = "true" {} " Record Holder:"
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
                        span.button.red.hover#record-delete style = "margin: 15px auto 0px" {"Delete Record"};
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

fn record_selector() -> Markup {
    html! {
        div.panel.fade {
            h2.underlined.pad {
                "Search record by ID"
            }
            p {
                "Records can be uniquely identified by ID. Entering a record's ID below will select it on the left (provided the record exists)"
            }
            form.flex.col#record-search-by-record-id-form novalidate = "" {
                p.info-red.output {}
                span.form-input#record-record-id {
                    label for = "id" {"Record ID:"}
                    input required = "" type = "number" name = "id" min = "0" style="width:93%"; // FIXME: I have no clue why the input thinks it's a special snowflake and fucks up its width, but I dont have the time to fix it
                    p.error {}
                }
                input.button.blue.hover type = "submit" style = "margin: 15px auto 0px;" value="Find by ID";
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
                (record_selector())
                (player_selector())
                (crate::view::demonlist::submit_panel())
            }
            (change_progress_dialog())
            (change_video_dialog())
            (change_holder_dialog())
            (change_demon_dialog(demons))
        }
    }
}

fn change_progress_dialog() -> Markup {
    html! {
        div.overlay.closable {
            div.dialog#record-progress-dialog {
                span.plus.cross.hover {}
                h2.underlined.pad {
                    "Change record progress:"
                }
                p style = "max-width: 400px"{
                    "Change the progress value of this record. Has to be between the demon's record requirement and 100 (inclusive)."
                }
                form.flex.col novalidate = "" {
                    p.info-red.output {}
                    p.info-green.output {}
                    span.form-input#record-progress-edit {
                        label for = "progress" {"Progress:"}
                        input name = "progress" type = "number" min = "0" max="100" required = "";
                        p.error {}
                    }
                    input.button.blue.hover type = "submit" style = "margin: 15px auto 0px;" value = "Edit";
                }
            }
        }
    }
}

fn change_video_dialog() -> Markup {
    html! {
        div.overlay.closable {
            div.dialog#record-video-dialog {
                span.plus.cross.hover {}
                h2.underlined.pad {
                    "Change video link:"
                }
                p style = "max-width: 400px"{
                    "Change the video link for this record. Note that as a list mod, you can leave the text field empty to remove the video from this record."
                }
                form.flex.col novalidate = "" {
                    p.info-red.output {}
                    p.info-green.output {}
                    span.form-input#record-video-edit {
                        label for = "video" {"Video link:"}
                        input name = "video" type = "url";
                        p.error {}
                    }
                    input.button.blue.hover type = "submit" style = "margin: 15px auto 0px;" value = "Edit";
                }
            }
        }
    }
}

fn change_holder_dialog() -> Markup {
    html! {
        div.overlay.closable {
            div.dialog#record-holder-dialog {
                span.plus.cross.hover {}
                h2.underlined.pad {
                    "Change record holder:"
                }
                div.flex.viewer {
                    (crate::view::filtered_paginator("record-holder-dialog-pagination", "/api/v1/players/"))
                    div {
                        p {
                            "Change the player associated with this record. If the player you want to change this record to already exists, search them up on the left and click them. In case the player does not exist, fill out only the text field on the right. This will prompt the server to create a new player."
                        }
                        form.flex.col novalidate = "" {
                            p.info-red.output {}
                            p.info-green.output {}
                            span.form-input#record-holder-name-edit {
                                label for = "player" {"Player name:"}
                                input name = "player" type="text" required = "";
                                p.error {}
                            }
                            input.button.blue.hover type = "submit" style = "margin: 15px auto 0px;" value = "Edit";
                        }
                    }
                }
            }
        }
    }
}

fn change_demon_dialog(demons: &[OverviewDemon]) -> Markup {
    html! {
        div.overlay.closable {
            div.dialog#record-demon-dialog {
                span.plus.cross.hover {}
                h2.underlined.pad {
                    "Change record demon:"
                }
                div.flex.col {
                    p {
                        "Change the demon associated with this record. Search up the demon this record should be associated with below. Then click it to modify the record"
                    }
                    div.dropdown-menu.js-search#edit-demon-record {
                        input type="text" style = "color: #444446; font-weight: bold;";
                        div.menu {
                           ul {
                                @for demon in demons {
                                    li.white.hover data-value = (demon.id) data-display = (demon.name) {b{"#"(demon.position) " - " (demon.name)} br; {"by "(demon.publisher)}}
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
