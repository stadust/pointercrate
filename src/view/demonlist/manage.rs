use crate::view::{account::AccountPage, paginator};
use maud::{html, Markup};

impl AccountPage {
    pub(in crate::view) fn record_page(&self) -> Markup {
        html! {
            div.m-center.flex.tab-content.container data-tab-id = "3" {
                div.left {
                    div.panel.fade#record-manager {
                        h2.underlined.pad {
                            "Record Manager"
                        }
                        div.flex {
                            (paginator("record-pagination", "/records/"))
                            div {
                                p#error-output style = "text-align: center" {
                                    "Click on a record on the left to get started!"
                                }
                                div#record-data style = "display:none" {
                                    div.flex.col {
                                        h3 style = "font-size:1.4em; overflow: hidden" { "Record #" i#record-id{}}

                                        iframe."ratio-16-9"#verification style="width:90%; margin: 15px 5%" allowfullscreen="" {"Verification Video"}
                                        div.stats-container.flex.space {
                                            span {
                                                b {
                                                    "Demon:"
                                                }
                                                br;
                                                span#demon {}
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
                                                span#progress {}
                                            }
                                            span {
                                                b {
                                                    "Submitter"
                                                }
                                                br;
                                                span#submitter {}
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                div.right {}
            }
        }
    }
}
