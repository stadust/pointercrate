use crate::components::{demon_dropdown, player_selection_dropdown};
use maud::{html, Markup};
use pointercrate_core::{localization::tr, trp};
use pointercrate_core_pages::{head::HeadLike as _, trp_html, PageFragment};
use pointercrate_demonlist::{config, demon::Demon, player::DatabasePlayer};

pub struct SubmitRecordPage {
    pub initial_holder: Option<DatabasePlayer>,
    /// Position of the demon initially selected
    pub initial_demon: Option<usize>,
    pub demons: Vec<Demon>,
}

impl From<SubmitRecordPage> for PageFragment {
    fn from(page: SubmitRecordPage) -> Self {
        PageFragment::new("Submit Record - Geometry Dash Demonlist", "Submit a record to the Demonlist")
            .module("/static/core/js/modules/form.js")
            .module("/static/demonlist/js/modules/demonlist.js")
            .module("/static/demonlist/js/demonlist.js")
            .stylesheet("/static/demonlist/css/submit.css")
            .body(page.body())
    }
}

impl SubmitRecordPage {
    fn body(&self) -> Markup {
        html! {
            form.panel.fade #submission-form novalidate = "" {
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
                    (demon_dropdown("id_demon", &self.demons, self.initial_demon))
                    p.error {}
                }
                h3 {
                    (tr("record-submission.holder"))
                }
                p {
                    (tr("record-submission.holder-info"))
                }
                span.form-input.flex.col data-type = "dropdown" {
                    (player_selection_dropdown("id_player", "/api/v1/players/", "name", "player", &self.initial_holder))
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

pub(crate) fn submit_record_panel(demon_position: Option<i16>) -> Markup {
    let search_params = match demon_position {
        Some(position) => format!("?demon={position:?}"),
        None => "".to_owned(),
    };

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
            a.blue.hover.button.js-scroll href=(format!("/demonlist/submit-record{search_params}")) {
                (tr("record-submission-panel.redirect"))
            }
        }
    }
}
