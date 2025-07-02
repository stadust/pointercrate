use crate::components::{demon_dropdown, player_selection_dropdown};
use maud::{html, Markup, PreEscaped, Render};
use pointercrate_core::{localization::tr, trp};
use pointercrate_demonlist::{config, demon::Demon};

pub struct RecordSubmitter<'a> {
    initially_visible: bool,
    demons: &'a [Demon],
}

impl<'a> RecordSubmitter<'a> {
    pub fn new(visible: bool, demons: &'a [Demon]) -> RecordSubmitter<'a> {
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
                        h2 { (tr("record-submission")) }
                    }
                    p.info-red.output {}
                    p.info-green.output {}
                    h3 {
                        (tr("record-submission.demon"))
                    }
                    p {
                        (trp!("record-submission.demon-info", ("list-size", config::extended_list_size())))
                    }
                    span.form-input data-type = "dropdown" {
                        (demon_dropdown("id_demon", self.demons.iter().filter(|demon| demon.base.position <= config::extended_list_size())))
                        p.error {}
                    }
                    h3 {
                        (tr("record-submission.holder"))
                    }
                    p {
                        (tr("record-submission.holder-info"))
                    }
                    span.form-input.flex.col data-type = "dropdown" {
                        (player_selection_dropdown("id_player", "/api/v1/players/", "name", "player"))
                        p.error {}
                    }
                    h3 {
                        (tr("record-submission.progress"))
                    }
                    p {
                        (tr("record-submission.progress-info"))
                    }
                    span.form-input.flex.col #id_progress {
                        input type = "number" name = "progress" required="" placeholder = (tr("record-submission.progress-placeholder")) min="0" max="100";
                        p.error {}
                    }
                    h3 {
                        (tr("record-submission.video"))
                    }
                    p {
                        (tr("record-submission.video-info"))
                        br {}

                        i { (tr("record-submission.note")) ": "  }
                        (tr("record-submission.video-note"))
                    }
                    span.form-input.flex.col #id_video {
                        input type = "url" name = "video" required = "" placeholder = (tr("record-submission.video-placeholder")) ;
                        p.error {}
                    }
                    h3 {
                        (tr("record-submission.raw-footage"))
                    }
                    p {
                        (tr("record-submission.raw-footage-info-a"))
                    }
                    p {
                        (tr("record-submission.raw-footage-info-b"))
                    }
                    p {
                        i { (tr("record-submission.note")) ": " } (tr("record-submission.raw-footage-note"))
                    }
                    span.form-input.flex.col #submit-raw-footage {
                        input type = "url"  name = "raw_footage" required = "" placeholder = "https://drive.google.com/file/d/.../view?usp=sharing" {}
                        p.error {}
                    }
                    h3 {
                        (tr("record-submission.notes"))
                    }
                    p {
                        (tr("record-submission.notes-info"))
                    }
                    span.form-input.flex.col #submit-note {
                        textarea name = "note" placeholder = (tr("record-submission.notes-placeholder")) {}
                        p.error {}
                    }
                    p {
                        (PreEscaped(trp!(
                            "record-submission.guidelines",
                            (
                                "guidelines-link",
                                html! {
                                    a.link href = "/guidelines" { (tr("record-submission.guidelines-link")) }
                                }.into_string()
                            )
                        )))
                    }
                    input.button.blue.hover type = "submit" style = "margin: 15px auto 0px;" value=(tr("record-submission.submit"));
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
                    (tr("record-submission-panel"))
                }
            }
            p {
                (tr("record-submission-panel.info"))
            }
            a.blue.hover.button.js-scroll data-destination = "submitter" data-reveal = "true" {
                (tr("record-submission-panel.redirect"))
            }
        }
    }
}
