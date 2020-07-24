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
            }
            div.right {

            }
        }
    }
}
