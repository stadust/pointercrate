use maud::{html, Markup, PreEscaped};

fn rules_panel() -> Markup {
    html! {
        did#rules.panel.fade.js-scroll-anim data-anim = "fade" {
            div.underlined {
                h2 {
                    "Rules:"
                }
            }
            ul.roman {
                li {
                    span {
                        "Anyone posting illegitimate recordings (hacked, cut, stolen, automated gameplay, no-clip, etc.) and passing them of as legit will have all their records removed from this list"
                    }
                }
                li {
                    span {
                        "Demons need to be rated to be included on this list"
                    }
                }
                li {
                    span {
                        "If you verified a level on this list, your record for it won't be included - You get points for your verification though"
                    }
                }
                li {
                    span {
                        "If a record has been added, it is legit and was either streamed or has a full video uploaded"
                    }
                }
                li {
                    span {
                        "The record holder must meet the percentage requirement in order to be added to the list for that level"
                    }
                }
                li {
                    span {
                        "Be polite about suggesting changes. We probably won't listed to you if you're rude or forceful about it"
                    }
                }
                li {
                    span {
                        "Being in a group in which people beat levels for the same channel, yet passing that channel of as being a single person's, can cause your records to be temporarily removed from this list"
                    }
                }
                li {
                    span {
                        "Records made using the FPS bypass are "
                        i { "not" }
                        "accepted"
                    }
                }
            }
        }
    }
}

fn submit_panel() -> Markup {
    html! {
        div#submit.panel.fade.js-scroll-anim data-anim = "fade" {
            div.underlined {
                h2 {
                    "Submit Records:"
                }
            }
            p {
                "Note: Please do not submit nonsense, it only makes it harder for us all and will get you banned. Also note that the form rejects duplicate submission"
            }
            a.blue.hover.button.slightly-rounded.js-scroll data-destination = "submitter" data-reveal = "true" {
                "Submit a record!"
            }
        }
    }
}

fn stats_viewer_panel() -> Markup {
    html! {
        div#stats.panel.fade.js-scroll-anim data-anim = "fade" {
            div.underlined {
                h2 {
                    "Stats Viewer"
                }
            }
            p {
                "Get a detailed overview of who completed the most, created the most demons or beat the hardest demons! There is even a leaderboard to compare yourself to the very best!"
            }
            a.blue.hover.button.slightly-rounded.js-scroll data-destination = "statsviewer" data-reveal = "true" {
                "Open the stats viewer!"
            }
        }
    }
}

fn discord_panel() -> Markup {
    html! {
        div.panel.fade.js-scroll-anim data-anim = "fade" {
            iframe#disccord style = "width: 100%; height: 400px;" allowtransparency="true" frameborder = "0" {}
            p {
                "Join the official demonlist discord server, where you can get in touch with the demonlist team!"
            }
        }
    }
}
