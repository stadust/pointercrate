use super::{url_helper, Page};
use crate::state::PointercrateState;
use actix_web::HttpRequest;
use maud::{html, Markup, PreEscaped};

#[derive(Debug)]
pub struct Homepage;

impl Page for Homepage {
    fn title(&self) -> String {
        "Home".to_owned()
    }

    fn description(&self) -> String {
        "Pointercrate is the home of the official Geometry Dash demonlist, a ranking of the hardest rated demons maintained by some of the game's most skilled players".to_owned()
    }

    fn scripts(&self) -> Vec<&str> {
        vec![]
    }

    fn stylesheets(&self) -> Vec<&str> {
        vec![]
    }

    fn body(&self, req: &HttpRequest<PointercrateState>) -> Markup {
        html! {
<<<<<<< HEAD
            div.tabbed style = "background-color: rgba(255,255,255, 0.3); display:flex; justify-content: center; height: 430px; font-size: 110%"{
                div.tab-display style="padding-top:50px; padding-bottom: 50px; display: flex; flex-flow: row; margin-left: 20px; justify-content: space-between; max-width: 1024px; width: 70%; align-items: center" {
                    div style ="display: flex; justify-content: space-between; flex-direction: column; width: 70%; height: 100%; padding-left: 30px" {
                        div style = "display: flex; flex-flow: column;"{
                            h1 style="text-align: left; margin-top: 0px" {
                                "Pointercrate"
                            }
                            h2 style="text-align: left" {
                                "Home of the official Geometry Dash Demonlist"
                            }
                            div.tab-content.tab-content-active data-tab-id ="1" {
                                "The pointercrate demonlist is the most popular ranking of the game's hardest demons with multiple thousand visitors each day! Even RobTop himself likes it!"
                            }
                            div.tab-content data-tab-id = "2" {
                                "The demonlist stats viewer assigns each player a score based on how many demons they've beaten and then ranks them, showing exactly who's the best!"
                            }
                            div.tab-content data-tab-id = "3" {
                                "Each submitted record on the demonlist is manually accepted or rejected by our competent list editors!"
                            }
                            div.tab-content data-tab-id = "4" {
                                "Thanks to our specialized way of connecting to the Geometry Dash servers we are able to display a whole range of information about the demons, including their description, download count and much more!"
                            }
                        }
                        div.tab-selection style="display: flex;align-items: center;justify-content: space-between;padding: 20px 0px; text-align: center"{
                            div.tab.tab-active.hover.scale data-tab-id="1" style="padding: 10px" {
                                h3 {
                                    "Ranking"
                                }
                                i class = "fa fa-list-ol fa-2x" aria-hidden="true" {}
                            }
                            div.tab.hover.scale data-tab-id="2" style="padding: 10px" {
                                h3 {
                                    "Stats Viewer"
                                }
                                i class = "fa fa-globe fa-2x" aria-hidden="true" {}
                            }
                            div.tab.hover.scale data-tab-id="3" style="padding: 10px" {
                                h3 {
                                    "Records"
                                }
                                i class = "fa fa-trophy fa-2x" aria-hidden="true" {}
                            }
                            div.tab.hover.scale data-tab-id="4" style="padding: 10px" {
                                h3 {
                                    "Informative"
                                }
                                i class = "fa fa-info fa-2x" aria-hidden="true" {}
                            }
                        }
                    }
                    a.big.blue.hover.button.js-scroll-anim data-anim="fade" href = {(url_helper::demon(req, 1))} style="height: 50px" {
                        "Check it out"(PreEscaped("&nbsp;&nbsp;&nbsp;"))
                        i.fa.fa-arrow-right aria-hidden="true" {}
                    }
=======
            div.panel.feature-panel.fade style="max-width:800px; margin-left:auto; margin-right: auto; font-size: 0.9em" {
                h3 style="font-size: 1.5em; margin-bottom: 0px;" {
                    "Home of the official Geometry Dash Demonlist!"
                }
                div.feature.js-scroll-anim.hover.scale data-anim="fade" {
                    i class = "fa fa-list-ol fa-2x blue2" aria-hidden="true" {}
                    h3.b2 {"Ranking"}
                    "Accurate ranking of the game's hardest demons, determined by some of its best players!"
                }
                div.feature.js-scroll-anim.hover.scale data-anim="fade" {
                    i class = "fa fa-trophy fa-2x blue2" aria-hidden="true" {}
                    h3.b2 {"Records"}
                    "Over 9000 records on currently 100 demons, with new ones getting submitted daily!"
                }
                div.feature.js-scroll-anim.hover.scale data-anim="fade" {
                    i class = "fa fa-clock-o fa-2x blue2" aria-hidden="true" {}
                    h3.b2 {"Up-to-date"}
                    "Thanks to a team of moderators all around the world, the list is updated nearly 24/7!"
>>>>>>> master
                }
            }
            div.center style="background: #0881c6;text-align: center;color: white;font-weight: bold;" {
                div.flex style="height: 50px; align-items: center" {
                    span { "Over 13 000 daily visitors!" }
                    span { "Over 150 ranked demons!" }
                    span { "Over 15 000 records!" }
                }
            }
            div.center style = "background-color: rgba(255,255,255, 0.3); display:flex; justify-content: center; height: 300px; font-size: 110%" {
                div style="padding-top:50px; padding-bottom: 50px; display: flex; flex-flow: row; margin-left: 20px; justify-content: space-between; max-width: 1024px; align-items: center" {
                    a.big.blue.hover.button.js-scroll-anim data-anim="fade" href = "https://github.com/stadust/pointercrate" style="height: 50px" target = "_blank"{
                        i.fa.fa-github aria-hidden="true" {}
                        (PreEscaped("&nbsp;&nbsp;&nbsp;"))
                        "To the repository"
                    }
                    div style = "display: flex; flex-flow: column; text-align: right; width: 70%"{
                        h2 style = "text-align: right"{ "Now on GitHub "}
                        h3 style = "text-align: right" { "The entirety of the pointercrate codebase can now be found on GitHub"}
                        p{"Found a bug on the website? Want to help with development? Or maybe you just want to find out how everything here works? Head over to the pointercrate GitHub repository!"}
                        p{"Even our custom Geometry Dash API wrapper, GDCF, can be found there!"}
                    }
                }
            }
            div.center style="background: #0881c6;text-align: center;color: white;font-weight: bold;" {
                div.flex style="height: 50px; align-items: center" {
                    span { "Written in Rust!" }
                    span { "Actively Maintained!" }
                    span { "I don't know what to write anymore!" }
                }
            }
        }
    }

    fn head(&self, _: &HttpRequest<PointercrateState>) -> Vec<Markup> {
        vec![html! {
            (PreEscaped(r#"
<style>
    .tab-active {
        color: #0881c6;
    }
</style>
<script type="application/ld+json">
  {
    "@context": "http://schema.org",
    "@type": "Organization",
    "name": "pointercrate",
    "description": "Pointercrate is the home of the official Geometry Dash demonlist, a ranking of the hardest rated demons maintained by some of the game's most skilled players",
    "url": "https://pointercrate.com/",
    "logo": "https://pointercrate.com/static/images/pointercrate2.png",
    "sameAs": [
      "https://twitter.com/demonlistgd",
      "https://www.youtube.com/channel/UCqI5feGZEqJRp6VcrP5gVyw"
    ]
  }
</script>
            "#))
        }]
    }
}
