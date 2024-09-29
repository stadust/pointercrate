use crate::components::{demon_dropdown, player_selection_dropdown};
use maud::{html, Markup, Render};
use pointercrate_demonlist::{config, demon::Demon};

pub struct RecordSubmitter<'a> {
    initially_visible: bool,
    demons: &'a [Demon],
}

impl RecordSubmitter<'_> {
    pub fn new(visible: bool, demons: &[Demon]) -> RecordSubmitter {
        RecordSubmitter {
            initially_visible: visible,
            demons,
        }
    }
}

impl Render for RecordSubmitter<'_> {
    fn render(&self) -> Markup {
        html! {
            section.panel.fade.closable #submitter style=(if !self.initially_visible {"display:none"} else {""}) {
                span.plus.cross.hover {}
                form #submission-form novalidate = "" {
                    div.underlined {
                        h2 {"Record Submission"}
                    }
                    p.info-red.output {}
                    p.info-green.output {}
                    h3 {
                        "Level:"
                    }
                    // Only demons in the top " (config::extended_list_size()) " are accepted. This excludes legacy demons!
                    p {
                        "The level the record was made on."
                    }
                    span.form-input data-type = "dropdown" {
                        (demon_dropdown("id_demon", self.demons.iter().filter(|demon| demon.base.position <= config::extended_list_size())))
                        p.error {}
                    }
                    h3 {
                        "Holder:"
                    }
                    p {
                        "The player holding the record. Start typing to see suggestions of existing players. If this is your first submission, write your name, as you wish it to appear on the website, into the text field (ignoring any suggestions)."
                    }
                    span.form-input.flex.col data-type = "dropdown" {
                        (player_selection_dropdown("id_player", "/api/v1/players/", "name", "player"))
                        p.error {}
                    }
                    span.form-input.flex.col style="display: none" #id_progress {
                        input type = "number" name = "progress" required="" value="100" placeholder = "e. g. '50', '98'" min="0" max="100";
                        p.error {}
                    }
                    h3 {
                        "(Optional) Enjoyment:"
                    }
                    p {
                        "A rating out of 10 based on how much you enjoyed this level. If it was an 8 out of 10, for example, write 8."
                    }
                    span.form-input.flex.col #id_enjoyment {
                        input type = "number" name = "enjoyment" placeholder = "e.g. 8 = 8/10" min="1" max="10";
                        p.error {}
                    }
                    h3 {
                        "Video: "
                    }
                    p {
                        "The video to be public on the site. This video can be edited if you also submit the unedited version below."
                        br {}

                        i { "Note: " }
                        "Please pay attention to only submit well-formed URLs!"
                    }
                    span.form-input.flex.col #id_video {
                        input type = "url" name = "video" required = "" placeholder = "e.g. 'https://youtu.be/EUBtwD-e2R0'" ;
                        p.error {}
                    }
                    h3 {
                        "(Optional) Unedited completion: "
                    }
                    p {
                        "If you want to edit your record's video, you must submit the unedited completion for review. The 'Video' field above will be the video public on the site, however."
                    }
                    p {
                        "Any personal information possibly contained within your unedited video (e.g. names, sensitive conversations) will be kept strictly confidential and will not be shared outside of the list team. Conversely, you acknowledge that you might inadvertently share such information. You have the right to request deletion of this video by contacting a list administrator."
                    }

                    span.form-input.flex.col #submit-raw-footage {
                        input type = "url"  name = "raw_footage" placeholder = "This does not need to be a YouTube link!" {}
                        p.error {}
                    }
                    h3 {
                        "Notes or comments: "
                    }
                    p {
                        "Provide any additional notes you'd like to pass on to the list moderator receiving your submission."
                    }
                    span.form-input.flex.col #submit-note {
                        textarea name = "note" placeholder = "e.g. this level SUCKS and it should be removed I HATE THIS LEVEL" {}
                        p.error {}
                    }
                    p {
                        "By submitting the record you acknowledge the " a.link href = "https://docs.google.com/document/d/1zW2tOWRi-qTxd2pM2FrParnVTzJjzRiGKIGGSJycKuI/edit?usp=sharing" {"submission guidelines"} "."
                    }
                    input.button.blue.hover type = "submit" style = "margin: 15px auto 0px;" value="Submit record";
                }
            }
        }
    }
}

pub(crate) fn submit_panel() -> Markup {
    html! {
        section #submit.panel.fade.js-scroll-anim data-anim = "fade" {
            div.underlined {
                h2 {
                    "Submit Records"
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
