use crate::components::{demon_dropdown, player_selection_dropdown};
use maud::{html, Markup, Render};
use pointercrate_core::{localization::tr, trp};
use pointercrate_core_pages::trp_html;
use pointercrate_demonlist::{config, demon::Demon, player::DatabasePlayer};

pub struct RecordSubmitter<'d, 'p> {
    initially_visible: bool,
    demons: &'d [Demon],
    initial_demon: Option<i16>,
    initial_holder: Option<&'p DatabasePlayer>,
}

impl<'d, 'p> RecordSubmitter<'d, 'p> {
    /// * `visible` - Show the record submitter.
    /// * `demons` - The  Demonlist.
    /// * `holder` - Player to preselect as the record holder. `None` to not preselect a player.
    /// * `demon` - Position of the demon in the demonlist to preselect. `None` to not preselect a demon.
    pub fn new(visible: bool, demons: &'d [Demon], holder: Option<&'p DatabasePlayer>, demon: Option<i16>) -> RecordSubmitter<'d, 'p> {
        RecordSubmitter {
            initially_visible: visible,
            demons,
            initial_demon: demon,
            initial_holder: holder,
        }
    }
}

impl Render for RecordSubmitter<'_, '_> {
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
                        (trp!("record-submission.demon-info", "list-size" = config::extended_list_size()))
                    }
                    span.form-input data-type = "dropdown" {
                        (demon_dropdown("id_demon", self.demons.iter().filter(|demon| demon.base.position <= config::extended_list_size()), self.initial_demon))
                        p.error {}
                    }
                    h3 {
                        (tr("record-submission.holder"))
                    }
                    p {
                        (tr("record-submission.holder-info"))
                    }
                    span.form-input.flex.col data-type = "dropdown" {
                        (player_selection_dropdown("id_player", "/api/v1/players/", "name", "player", self.initial_holder))
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
                        (trp_html!(
                            "record-submission.guidelines",
                            "guidelines-link" = html! {
                                a.link href = "/guidelines" { (tr("record-submission.guidelines-link")) }
                            }
                        ))
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
