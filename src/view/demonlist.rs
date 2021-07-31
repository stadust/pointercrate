pub use self::{
    demon_page::{demon_permalink, page},
    overview::{index, overview_demons, OverviewDemon},
    statsviewer::{
        heatmap::heatmap_css, individual::stats_viewer as individual_statsviewer, nationbased::stats_viewer as nation_statsviewer,
    },
};
use crate::{
    config,
    model::{demonlist::demon::Demon, nationality::Nationality},
};
use maud::{html, Markup, PreEscaped, Render};

mod demon_page;
mod overview;
mod statsviewer;

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
            a href = {"/demonlist/permalink/" (demon.id) "/"} {
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

pub fn demon_dropdown<'a>(dropdown_id: &str, demons: impl Iterator<Item = &'a OverviewDemon>) -> Markup {
    html! {
        div.dropdown-menu.js-search#(dropdown_id) {
            div {
                input type = "text" name = "demon" required="" autocomplete="off";
            }
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

pub fn player_selection_dialog(dialog_id: &str, headline: &str, description: &str, button_text: &str) -> Markup {
    html! {
        div.overlay.closable {
            div.dialog#(dialog_id) {
                span.plus.cross.hover {}
                h2.underlined.pad {
                    (headline)
                }
                div.flex.viewer {
                    (crate::view::filtered_paginator(&format!("{}-pagination", dialog_id), "/api/v1/players/"))
                    div {
                        p {
                            (description)
                        }
                        form.flex.col novalidate = "" {
                            p.info-red.output {}
                            p.info-green.output {}
                            span.form-input#{(dialog_id)"-input"} {
                                label for = "player" {"Player name:"}
                                input name = "player" type="text" required = "";
                                p.error {}
                            }
                            input.button.blue.hover type = "submit" style = "margin: 15px auto 0px;" value = (button_text);
                        }
                    }
                }
            }
        }
    }
}

pub(super) fn submission_panel(demons: &[OverviewDemon], visible: bool) -> Markup {
    html! {
        section.panel.fade.closable#submitter style=(if !visible {"display:none"} else {""}) {
            span.plus.cross.hover {}
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
                span.form-input data-type = "dropdown" {
                    (demon_dropdown("id_demon", demons.iter().filter(|demon| demon.position <= config::extended_list_size())))
                    p.error {}
                }
                h3 {
                    "Holder:"
                }
                p {
                    "The holder of the record. Please enter the holders Geometry Dash name here, even if their YouTube name differs! Click the pencil to select a player!"
                }
                span.form-input.flex.col#id_player data-type = "html" data-target-id = "selected-holder" data-default = "None Selected" {
                    span {
                        b {
                            i.fa.fa-pencil-alt.clickable#record-submitter-holder-pen aria-hidden = "true" {}
                            " "
                        }
                        i#selected-holder data-name = "player" {"None Selected"}
                    }
                    p.error {}
                }
                h3 {
                    "Progress:"
                }
                p {
                    "The progress made as percentage. Only values greater than or equal to the demons record requirement and smaller than or equal to 100 are accepted!"
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
                    "Provide any additional notes you'd like to pass on to the list moderator receiving your submission. In particular, any required " b { "raw footage"} " goes here. Any personal information possibly contained within raw footage (e.g. names, sensitive conversations) will be kept strictly confidential and will not be shared outside of the demonlist team. Conversely, you acknowledge that you might inadvertently share such information by providing raw footage. You have the right to request deletion of your record note by contacting a list administrator."
                }
                span.form-input.flex.col#submit-note {
                    textarea name = "note" placeholder = "Your dreams and hopes for this record... or something like that" {}
                    p.error {}
                }
                p {
                    "By submitting the record you acknowledge the " a.link href = "/guidelines" {"submission guidelines"} "."
                }
                input.button.blue.hover type = "submit" style = "margin: 15px auto 0px;" value="Submit record";
            }
        }
        (player_selection_dialog(
            "submission-holder-dialog",
            "Select player:",
            "To select the player holding this record, search them up on the left to see if they already have records on the list and click them. In case the player does not exist, fill out only the text field on the right.",
            "Select"
        ))
    }
}
fn sidebar_ad() -> Markup {
    html! {
        section.panel.fade.js-scroll-anim data-anim = "fade" style = "order: 1; padding: 0px; border: 0" {
            (PreEscaped(format!(r#"
            <script async src="https://pagead2.googlesyndication.com/pagead/js/adsbygoogle.js?client={0}"
     crossorigin="anonymous"></script>
<!-- Demonlist Sidebar Ad -->
<ins class="adsbygoogle"
     style="display:block"
     data-ad-client="{0}"
     data-ad-slot="2559641548"
     data-ad-format="auto"
     data-full-width-responsive="true"></ins>
<script>
     (adsbygoogle = window.adsbygoogle || []).push({{}});
</script>
            "#, config::adsense_publisher_id())))
        }
    }
}

fn besides_sidebar_ad() -> Markup {
    html! {
        div#outofboundsad style="margin-left: calc(45% + 1072px/2);position: fixed;padding-left: 15px;padding-top: 15px; max-width: 200px" {
            (PreEscaped(format!(r#"
                <script async src="https://pagead2.googlesyndication.com/pagead/js/adsbygoogle.js?client={0}"
     crossorigin="anonymous"></script>
<!-- Demonlist Sidebar Ad #2 -->
<ins class="adsbygoogle"
     style="display:block"
     data-ad-client="{0}"
     data-ad-slot="3380750697"
     data-ad-format="auto"
     data-full-width-responsive="true"></ins>
<script>
     (adsbygoogle = window.adsbygoogle || []).push({{}});
</script>
            "#, config::adsense_publisher_id())))
        }
    }
}

fn rules_panel() -> Markup {
    html! {
        section#rules.panel.fade.js-scroll-anim data-anim = "fade" {
            h2.underlined.pad.clickable {
                "Guidelines:"
            }
            p {
                "All demonlist operations are carried out in accordance to our guidelines. Be sure to check them before submitting a record to ensure a flawless experience!"
            }
            a.blue.hover.button href = "/guidelines/" {
                "Read the guidelines!"
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
            a.blue.hover.button#show-stats-viewer href = "/demonlist/statsviewer/ "{
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

impl Render for Nationality {
    fn render(&self) -> Markup {
        html! {
            span.flag-icon.{"flag-icon-"(self.iso_country_code.to_lowercase())} title = (self.nation) {}
        }
    }
}
