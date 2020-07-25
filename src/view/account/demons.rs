use crate::view::filtered_paginator;
use maud::{html, Markup, PreEscaped};

pub(super) fn page() -> Markup {
    html! {
        div.m-center.flex.tab-content.container data-tab-id = "5"{
            div.left {
                div.panel.fade {
                    h2.underlined.pad {
                        "Demon Manager"
                    }
                    div.flex.viewer {
                        (filtered_paginator("demon-pagination", "/api/v2/demons/listed/"))
                        p.viewer-welcome {
                            "Click on a demon on the left to get started!"
                        }

                        div.viewer-content {
                            div.flex.col{
                                h3 style = "font-size:1.1em; margin: 10px 0" {
                                    "Demon #"
                                    i#demon-demon-id {}
                                    " - "
                                    i.fa.fa-pencil.clickable#demon-name-pen aria-hidden = "true" {} (PreEscaped("&nbsp;")) i#demon-demon-name {}
                                }

                                iframe."ratio-16-9"#demon-video style="width:90%; margin: 15px 5%" allowfullscreen="" {"Verification Video"}
                                p.info-red.output style = "margin: 10px" {}
                                p.info-green.output style = "margin: 10px" {}
                                div.stats-container.flex.space  {
                                    span{
                                        b {
                                            i.fa.fa-pencil.clickable#demon-video-pen aria-hidden = "true" {} " Verification Video:"
                                        }
                                        br;
                                        a.link#demon-video-link target = "_blank" {}
                                    }
                                }
                                div.stats-container.flex.space  {
                                    span{
                                        b {
                                            i.fa.fa-pencil.clickable#demon-position-pen aria-hidden = "true" {} " Position:"
                                        }
                                        br;
                                        span#demon-position {}
                                    }
                                    span{
                                        b {
                                            i.fa.fa-pencil.clickable#demon-requirement-pen aria-hidden = "true" {} " Requirement:"
                                        }
                                        br;
                                        span#demon-requirement {}
                                    }
                                }
                                div.stats-container.flex.space  {
                                    span{
                                        b {
                                            i.fa.fa-pencil.clickable#demon-publisher-pen aria-hidden = "true" {} " Publisher:"
                                        }
                                        br;
                                        span#demon-publisher {}
                                    }
                                    span{
                                        b {
                                            i.fa.fa-pencil.clickable#demon-verifier-pen aria-hidden = "true" {} " Verifier:"
                                        }
                                        br;
                                        span#demon-verifier {}
                                    }
                                }
                                div.stats-container.flex.space  {
                                    span{
                                        b {
                                            "Creators:"
                                        }
                                        br;
                                        span#demon-creators {}
                                        (PreEscaped("&nbsp;"))
                                        i.fa.fa-plus.clickable#demon-position-pen aria-hidden = "true" {}
                                    }
                                }
                            }
                        }
                    }
                }
                div style="height: 50px" {} // to make sure that the footer doesnt float. if it floats, the user page is the only one without a scrollbar at the right, which causes jumpyness when switching tabs.
            }
            div.right {

            }
            (change_name_dialog())
            (change_position_dialog())
            (change_requirement_dialog())
            (change_video_dialog())
            (change_verifier_dialog())
            (change_publisher_dialog())
        }
    }
}

fn change_name_dialog() -> Markup {
    html! {
        div.overlay.closable {
            div.dialog#demon-name-dialog {
                span.plus.cross.hover {}
                h2.underlined.pad {
                    "Change demon name:"
                }
                p style = "max-width: 400px"{
                    "Change the name of this demon. Multiple demons with the same name ARE supported!"
                }
                form.flex.col novalidate = "" {
                    p.info-red.output {}
                    p.info-green.output {}
                    span.form-input#demon-name-edit {
                        label for = "name" {"Name:"}
                        input name = "name" type = "text" required = "";
                        p.error {}
                    }
                    input.button.blue.hover type = "submit" style = "margin: 15px auto 0px;" value = "Edit";
                }
            }
        }
    }
}

fn change_requirement_dialog() -> Markup {
    html! {
        div.overlay.closable {
            div.dialog#demon-requirement-dialog {
                span.plus.cross.hover {}
                h2.underlined.pad {
                    "Change demon requirement:"
                }
                p style = "max-width: 400px"{
                    "Change the record requirement for this demon. Has be lie between 0 and and 100 (inclusive)."
                }
                form.flex.col novalidate = "" {
                    p.info-red.output {}
                    p.info-green.output {}
                    span.form-input#demon-requirement-edit {
                        label for = "requirement" {"Requirement:"}
                        input name = "requirement" type = "number" min = "0" max="100" required = "";
                        p.error {}
                    }
                    input.button.blue.hover type = "submit" style = "margin: 15px auto 0px;" value = "Edit";
                }
            }
        }
    }
}

fn change_position_dialog() -> Markup {
    html! {
        div.overlay.closable {
            div.dialog#demon-position-dialog {
                span.plus.cross.hover {}
                h2.underlined.pad {
                    "Change demon position:"
                }
                p style = "max-width: 400px"{
                    "Change the position of this demon. Has be be greater than 0 and be at most the current list size."
                }
                form.flex.col novalidate = "" {
                    p.info-red.output {}
                    p.info-green.output {}
                    span.form-input#demon-position-edit {
                        label for = "position" {"Position:"}
                        input name = "position" type = "number" min = "1" required = "";
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
            div.dialog#demon-video-dialog {
                span.plus.cross.hover {}
                h2.underlined.pad {
                    "Change verification video link:"
                }
                p style = "max-width: 400px"{
                    "Change the verification video link for this record. Leave empty to remove the verification video. ."
                }
                form.flex.col novalidate = "" {
                    p.info-red.output {}
                    p.info-green.output {}
                    span.form-input#demon-video-edit {
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

fn change_verifier_dialog() -> Markup {
    html! {
        div.overlay.closable {
            div.dialog#demon-verifier-dialog {
                span.plus.cross.hover {}
                h2.underlined.pad {
                    "Change demon verifier:"
                }
                div.flex.viewer {
                    (crate::view::filtered_paginator("demon-verifier-dialog-pagination", "/api/v1/players/"))
                    div {
                        p {
                            "Change the verifier of this demon. If the player you want to change the verifier to already exists, search them up on the left and click them. In case the player does not exist, fill out only the text field on the right. This will prompt the server to create a new player."
                        }
                        form.flex.col novalidate = "" {
                            p.info-red.output {}
                            p.info-green.output {}
                            span.form-input#demon-verifier-name-edit {
                                label for = "verifier" {"Verifier name:"}
                                input name = "verifier" type="text" required = "";
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

fn change_publisher_dialog() -> Markup {
    html! {
        div.overlay.closable {
            div.dialog#demon-publisher-dialog {
                span.plus.cross.hover {}
                h2.underlined.pad {
                    "Change demon publisher:"
                }
                div.flex.viewer {
                    (crate::view::filtered_paginator("demon-publisher-dialog-pagination", "/api/v1/players/"))
                    div {
                        p {
                            "Change the publisher of this demon. If the player you want to change the publisher to already exists, search them up on the left and click them. In case the player does not exist, fill out only the text field on the right. This will prompt the server to create a new player."
                        }
                        form.flex.col novalidate = "" {
                            p.info-red.output {}k
                            p.info-green.output {}
                            span.form-input#demon-publisher-name-edit {
                                label for = "publisher" {"Publisher name:"}
                                input name = "publisher" type="text" required = "";
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
