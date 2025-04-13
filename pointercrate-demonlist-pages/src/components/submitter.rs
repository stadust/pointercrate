use crate::components::{demon_dropdown, player_selection_dropdown};
use maud::{html, Markup, PreEscaped, Render};
use pointercrate_core::localization::{ftr, tr};
use pointercrate_demonlist::{config, demon::Demon};
use unic_langid::LanguageIdentifier;

pub struct RecordSubmitter<'a> {
    initially_visible: bool,
    demons: &'a [Demon],
    lang: &'static LanguageIdentifier,
}

impl<'a> RecordSubmitter<'a> {
    pub fn new(visible: bool, demons: &'a [Demon], lang: &'static LanguageIdentifier) -> RecordSubmitter<'a> {
        RecordSubmitter {
            initially_visible: visible,
            demons,
            lang,
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
                        h2 { (tr(&self.lang, "record-submission")) }
                    }
                    p.info-red.output {}
                    p.info-green.output {}
                    h3 {
                        (tr(&self.lang, "record-submission.demon")) ":"
                    }
                    p {
                        (ftr(&self.lang, "record-submission.demon-info", &vec![("list-size", config::extended_list_size())]))
                    }
                    span.form-input data-type = "dropdown" {
                        (demon_dropdown("id_demon", self.demons.iter().filter(|demon| demon.base.position <= config::extended_list_size())))
                        p.error {}
                    }
                    h3 {
                        (tr(&self.lang, "record-submission.holder")) ":"
                    }
                    p {
                        (tr(&self.lang, "record-submission.holder-info"))
                    }
                    span.form-input.flex.col data-type = "dropdown" {
                        (player_selection_dropdown("id_player", "/api/v1/players/", "name", "player"))
                        p.error {}
                    }
                    h3 {
                        (tr(&self.lang, "record-submission.progress")) ":"
                    }
                    p {
                        (tr(&self.lang, "record-submission.progress-info"))
                    }
                    span.form-input.flex.col #id_progress {
                        input type = "number" name = "progress" required="" placeholder = (tr(&self.lang, "record-submission.progress-placeholder")) min="0" max="100";
                        p.error {}
                    }
                    h3 {
                        (tr(&self.lang, "record-submission.video")) ":"
                    }
                    p {
                        (tr(&self.lang, "record-submission.video-info"))
                        br {}

                        i { (tr(&self.lang, "record-submission.note")) ":" }
                        (tr(&self.lang, "record-submission.video-note"))
                    }
                    span.form-input.flex.col #id_video {
                        input type = "url" name = "video" required = "" placeholder = (tr(&self.lang, "record-submission.video-placeholder")) ;
                        p.error {}
                    }
                    h3 {
                        (tr(&self.lang, "record-submission.raw-footage")) ":"
                    }
                    p {
                        (tr(&self.lang, "record-submission.raw-footage-info-a"))
                    }
                    p {
                        (tr(&self.lang, "record-submission.raw-footage-info-b"))
                    }
                    p {
                        i {(tr(&self.lang, "record-submission.note")) ":"} (tr(&self.lang, "record-submission.raw-footage-note"))
                    }
                    span.form-input.flex.col #submit-raw-footage {
                        input type = "url"  name = "raw_footage" required = "" placeholder = (tr(&self.lang, "record-submission.raw-footage-placeholder")) {}
                        p.error {}
                    }
                    h3 {
                        (tr(&self.lang, "record-submission.notes")) ":"
                    }
                    p {
                        (tr(&self.lang, "record-submission.notes-info"))
                    }
                    span.form-input.flex.col #submit-note {
                        textarea name = "note" placeholder = (tr(&self.lang, "record-submission.notes-placeholder")) {}
                        p.error {}
                    }
                    p {
                        (PreEscaped(ftr(
                            &self.lang,
                            "record-submission.guidelines",
                            &vec![
                                (
                                    "guidelines-redirect",
                                    &format!(r#"<a class="link" href="/guidelines">{}</a>"#,
                                    tr(&self.lang, "record-submission.guidelines-redirect"))
                                )
                            ]
                        )))
                    }
                    input.button.blue.hover type = "submit" style = "margin: 15px auto 0px;" value=(tr(&self.lang, "record-submission.submit"));
                }
            }
        }
    }
}

pub(crate) fn submit_panel(lang: &'static LanguageIdentifier) -> Markup {
    html! {
        section #submit.panel.fade.js-scroll-anim data-anim = "fade" {
            div.underlined {
                h2 {
                    (tr(lang, "record-submission-panel"))
                }
            }
            p {
                (tr(lang, "record-submission-panel.info"))
            }
            a.blue.hover.button.js-scroll data-destination = "submitter" data-reveal = "true" {
                (tr(lang, "record-submission-panel.redirect"))
            }
        }
    }
}
