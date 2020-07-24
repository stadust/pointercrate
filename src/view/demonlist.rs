pub use self::{
    demon_page::page,
    overview::{index, overview_demons, OverviewDemon},
};
use crate::{
    config,
    model::{demonlist::demon::Demon, nationality::Nationality},
};
use maud::{html, Markup, PreEscaped};

mod demon_page;
mod overview;

struct ListSection {
    name: &'static str,
    description: &'static str,
    id: &'static str,
    numbered: bool,
}

static MAIN_SECTION: ListSection = ListSection {
    name: "Main List",
    description: "The main section of the Demonlist. These demons are the hardest rated levels in the game. Records are accepted above a \
                  given threshold and award a large amount of points!",
    id: "mainlist",
    numbered: true,
};

static EXTENDED_SECTION: ListSection = ListSection {
    name: "Extended List",
    description: "These are demons that dont qualify for the main section of the list, but are still of high relevance. Only 100% records \
                  are accepted for these demons! Note that non-100% that were submitted/approved before a demon fell off the main list \
                  will be retained",
    id: "extended",
    numbered: true,
};

static LEGACY_SECTION: ListSection = ListSection {
    name: "Legacy List",
    description: "These are demons that used to be on the list, but got pushed off as new demons were added. They are here for nostalgic \
                  reasons. This list is in no order whatsoever and will not be maintained any longer at all. This means no new records \
                  will be added for these demons.",
    id: "legacy",
    numbered: false,
};

fn dropdowns(all_demons: &[OverviewDemon], current: Option<&Demon>) -> Markup {
    let (main, extended, legacy) = if all_demons.len() < config::list_size() as usize {
        (&all_demons[..], Default::default(), Default::default())
    } else {
        let (extended, legacy) = if all_demons.len() < config::extended_list_size() as usize {
            (&all_demons[config::list_size() as usize..], Default::default())
        } else {
            (
                &all_demons[config::list_size() as usize..config::extended_list_size() as usize],
                &all_demons[config::extended_list_size() as usize..],
            )
        };

        (&all_demons[..config::list_size() as usize], extended, legacy)
    };

    html! {
        nav.flex.wrap.m-center.fade#lists style="text-align: center;" {
            // The drop down for the main list:
            (dropdown(&MAIN_SECTION, main, current))
            // The drop down for the extended list:
            (dropdown(&EXTENDED_SECTION, extended, current))
            // The drop down for the legacy list:
            (dropdown(&LEGACY_SECTION, legacy, current))
        }
    }
}

fn dropdown(section: &ListSection, demons: &[OverviewDemon], current: Option<&Demon>) -> Markup {
    let format = |demon: &OverviewDemon| -> Markup {
        html! {
            a href = {"/demonlist/" (demon.position)} {
                @if section.numbered {
                    {"#" (demon.position) " - " (demon.name)}
                    br ;
                    i {
                        (demon.publisher)
                    }
                }
                @else {
                    {(demon.name)}
                    br ;
                    i {
                        (demon.publisher)
                    }
                }
            }
        }
    };

    html! {
        div {
            div.button.white.hover.no-shadow.js-toggle data-toggle-group="0" onclick={"javascript:void(DropDown.toggleDropDown('" (section.id) "'))"} {
                (section.name)
            }

            div.see-through.fade.dropdown#(section.id) {
                div.search.js-search.seperated style = "margin: 10px" {
                    input placeholder = "Filter..." type = "text" {}
                }
                p style = "margin: 10px" {
                    (section.description)
                }
                ul.flex.wrap.space {
                    @for demon in demons {
                        @match current {
                            Some(current) if current.base.position == demon.position =>
                                li.hover.white.active title={"#" (demon.position) " - " (demon.name)} {
                                    (format(demon))
                                },
                            _ =>
                                li.hover.white title={"#" (demon.position) " - " (demon.name)} {
                                    (format(demon))
                                }
                        }
                    }
                }
            }
        }
    }
}

pub(super) fn submission_panel() -> Markup {
    html! {
        section.panel.fade.closable#submitter style = "display: none" {
            span.plus.cross.hover {}
            div.flex {
                form#submission-form novalidate = "" {
                    div.underlined {
                        h2 {"Record Submission"}
                    }
                    p.info-red.output {}
                    p.info-green.output {}
                    h3 {
                        "Demon:"
                    }
                    p {
                        "The demon the record was made on. Only demons in the top " (config::extended_list_size()) " are accepted. This excludes legacy demons!"
                    }
                    span.form-input.flex.col#id_demon {
                        input type = "text" name = "demon" required="" placeholder = "e. g. 'Bloodbath', 'Yatagarasu'" ;
                        p.error {}
                    }
                    h3 {
                        "Holder:"
                    }
                    p {
                        "The holder of the record. Please enter the holders Geometry Dash name here, even if their YouTube name differs!"
                    }
                    span.form-input.flex.col#id_player {
                        input type = "text" name = "player" required="" placeholder="e. g. 'Slypp, 'KrmaL'" maxlength="50" ;
                        p.error {}
                    }
                    h3 {
                        "Progress:"
                    }
                    p {
                        "The progress made as percentage. Only values greater than the demons record requirement and smaller than or equal to 100 are accepted!"
                    }
                    span.form-input.flex.col#id_progress {
                        input type = "number" name = "progress" required="" placeholder = "e. g. '50', '98'" min="0" max="100";
                        p.error {}
                    }
                    h3 {
                        "Video: "
                    }
                    p {
                        "A proof video of the legitimacy of the given record. If the record was achieved on stream, but wasn't uploaded anywhere else, please provide a twitch link to that stream."
                        br {}

                        i { "Note: " }
                        "Please pay attention to only submit well-formed URLs!"
                    }
                    span.form-input.flex.col#id_video {
                        input type = "url" name = "video" required = "" placeholder = "e.g. 'https://youtu.be/cHEGAqOgddA'" ;
                        p.error {}
                    }
                    h3 {
                        "Notes or comments: "
                    }
                    p {
                        "Provide any additional notes you'd like to pass on to the list moderator receiving your submission."

                    }
                    span.form-input.flex.col#submit-note {
                        textarea name = "note" placeholder = "Your dreams and hopes for this records... or something like that" {}
                        p.error {}
                    }
                    input.button.blue.hover type = "submit" style = "margin: 15px auto 0px;" value="Submit record";
                }
            }
        }
    }
}

fn stats_viewer(nations: &[Nationality]) -> Markup {
    html! {
        section.panel.fade.closable#statsviewer style = "display:none" {
            span.plus.cross.hover {}
            h2.underlined.pad {
                "Stats Viewer - "
                (super::dropdown("International",
                    html! {
                        li.white.hover.underlined data-value = "International" data-display = "International" {
                            span.em.em-world_map {}
                            (PreEscaped("&nbsp;"))
                            b {"WORLD"}
                            br;
                            span style = "font-size: 90%; font-style: italic" { "International" }
                        }
                    },
                    nations.iter().map(|nation| html! {
                        li.white.hover data-value = {(nation.iso_country_code)} data-display = {(nation.nation)} {
                            span class = {"flag-icon flag-icon-" (nation.iso_country_code.to_lowercase())} {}
                            (PreEscaped("&nbsp;"))
                            b {(nation.iso_country_code)}
                            br;
                            span style = "font-size: 90%; font-style: italic" {(nation.nation)}
                        }
                    })
                ))
            }
            div.flex.viewer {
                (super::filtered_paginator("stats-viewer-pagination", "/api/v1/players/ranking/"))
                p.viewer-welcome {
                    "Click on a player's name on the left to get started!"
                }
                div.viewer-content {
                    div {
                        div.flex.col {
                            h3#player-name style = "font-size:1.4em; overflow: hidden" {}
                            div.stats-container.flex.space {
                                span {
                                    b {
                                        "List demons completed:"
                                    }
                                    br;
                                    span#amount-beaten {}
                                }
                                span {
                                    b {
                                        "Legacy demons completed:"
                                    }
                                    br;
                                    span#amount-legacy {}
                                }
                                span {
                                    b {
                                        "Demonlist score:"
                                    }
                                    br;
                                    span#score {}
                                }
                            }
                            div.stats-container.flex.space {
                                span {
                                    b {
                                        "Demonlist rank:"
                                    }
                                    br;
                                    span#rank {}
                                }
                                span {
                                    b {
                                        "Hardest demon:"
                                    }
                                    br;
                                    span#hardest {}
                                }
                            }
                            div.stats-container.flex.space {
                                span {
                                    b {
                                        "Demons completed:"
                                    }
                                    br;
                                    span#beaten {}
                                }
                            }
                            div.stats-container.flex.space {
                                span {
                                    b {
                                        "List demons created:"
                                    }
                                    br;
                                    span#created {}
                                }
                                span {
                                    b {
                                        "List demons published:"
                                    }
                                    br;
                                    span#published {}
                                }
                                span {
                                    b {
                                        "List demons verified:"
                                    }
                                    br;
                                    span#verified {}
                                }
                            }
                            div.stats-container.flex.space {
                                span {
                                    b {
                                        "Progress on:"
                                    }
                                    br;
                                    span#progress {}
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn rules_panel() -> Markup {
    html! {
        section#rules.panel.fade.js-scroll-anim.js-collapse data-anim = "fade" {
            h2.underlined.pad.clickable {
                "Rules:"
                span.arrow.hover {}
            }
            ul.roman.js-collapse-content style="display:none" {
                h3 {
                    "Demon rules:"
                }
                li {
                    span {
                        "Demons need to be rated to be included on this list"
                    }
                }
                li {
                    span {
                        "List demons that receive a hacked update changing difficulty will be moved to the legacy section of the list. Alternatively, if a demon gets a hacked update before being list-worthy, it will not get added. However, a demon whose original verification was hacked will still get on the list."
                    }
                }
                h3 {
                    "Submission rules:"
                }
                li {
                    span {
                        "Records must be legitimate and either uploaded on YouTube, Vimeo, Bilibili or streamed to be added to the list."
                    }
                }
                li {
                    span {
                        " Anyone posting illegitimate recordings and passing them off as legit will have their records removed from the list. Illegitimate records include, but aren't limited to, speedhacks, noclip, auto, nerfs, macros, etc."
                    }
                }
                li {
                    span {
                        "Records on a level must be in normal mode and on the live version of the level or on an appropriate bug fixed/low detail copy of said level. Please refer to the bugfix and LDM guidelines."
                    }
                }
                li {
                    span {
                        "The record holder must meet the percentage requirement of a level in order to be added to the list for said level."
                    }
                }
                h3 {
                    "General guidelines:"
                }
                li {
                    span {
                        "Verifications are not counted as records on the list, but still award points."
                    }
                }
                li {
                    span {
                        "Being in a group in which people beat levels for the same channel will cause your records to be temporarily removed from the list."
                    }
                }
                h3 {
                    "Bugfix and LDM guidelines:"
                }
                li {
                    span {
                        "Records using a level's built-in LDM are always eligible. "
                    }
                }
                li {
                    span {
                        "Records on appropriate LDM copies of levels are eligible. Please take contact with a List Moderator if you are unsure of which decorations can or cannot be removed. Generally speaking, a LDM copy should not remove decorations that obstruct the player's vision, blind transitions, flashes or boss fights, for example. Referring to the first guideline, if the previously stated decorations are removed in a level's built-in LDM though, it is perfectly fine to use it."
                    }
                }
                li {
                    span {
                        "Records on appropriate bugfix copies of levels for different refresh rates are eligible. Please take contact with a List Moderator if you are unsure of what is or isn't a bug."
                    }
                }
            }
        }
    }
}

pub(super) fn submit_panel() -> Markup {
    html! {
        section#submit.panel.fade.js-scroll-anim data-anim = "fade" {
            div.underlined {
                h2 {
                    "Submit Records:"
                }
            }
            p {
                "Note: Please do not submit nonsense, it only makes it harder for us all and will get you banned. Also note that the form rejects duplicate submissions."
            }
            a.blue.hover.button.js-scroll data-destination = "submitter" data-reveal = "true" {
                "Submit a record!"
            }
        }
    }
}

fn stats_viewer_panel() -> Markup {
    html! {
        section#stats.panel.fade.js-scroll-anim data-anim = "fade" {
            div.underlined {
                h2 {
                    "Stats Viewer:"
                }
            }
            p {
                "Get a detailed overview of who completed the most, created the most demons or beat the hardest demons! There is even a leaderboard to compare yourself to the very best!"
            }
            a.blue.hover.button.js-scroll#show-stats-viewer data-destination = "statsviewer" data-reveal = "true" {
                "Open the stats viewer!"
            }
        }
    }
}

fn discord_panel() -> Markup {
    html! {
        section.panel.fade.js-scroll-anim#discord data-anim = "fade" {
            iframe.js-delay-attr style = "width: 100%; height: 400px;" allowtransparency="true" frameborder = "0" data-attr = "src" data-attr-value = "https://discordapp.com/widget?id=395654171422097420&theme=light" {}
            p {
                "Join the official Demonlist discord server, where you can get in touch with the demonlist team!"
            }
        }
    }
}
