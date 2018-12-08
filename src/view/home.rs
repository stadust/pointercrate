use super::{Page, DEMONLIST};
use maud::{html, Markup, PreEscaped};

#[derive(Debug)]
pub struct Homepage;

impl Page for Homepage {
    fn title(&self) -> &str {
        "Home"
    }

    fn description(&self) -> &str {
        "Pointercrate is the home of the official Geometry Dash demonlist, a ranking of the hardest rated demons maintained by some of the game's most skilled players"
    }

    fn scripts(&self) -> Vec<&str> {
        vec![]
    }

    fn stylesheets(&self) -> Vec<&str> {
        vec![]
    }

    fn body(&self) -> Markup {
        html!{
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
                }
                div.feature.js-scroll-anim.hover.scale data-anim="fade" {
                    i class = "fa fa-info fa-2x blue2" aria-hidden="true" {}
                    h3.b2 {"Information"}
                    "A whole range of information associated with each demon is displayed, including, but not limited to, its verification video and level description!"
                }
                div.feature.js-scroll-anim.hover.scale data-anim="fade" {
                    i class = "fa fa-globe fa-2x blue2" aria-hidden="true" {}
                    h3.b2 {"States Viewer"}
                    "Compare yourself to the very best players all around the world in the stats viewer, where a score is calculated based on all your beaten demons!"
                }
                a.big.blue.hover.button.slightly-round.fade.js-scroll-anim data-anim="fade" href = {(DEMONLIST)} style = "margin: 20px 40% 0px; min-width: 100px" {
                    "Check it out"(PreEscaped("&nbsp;&nbsp;&nbsp;"))
                    i.fa.fa-arrow-right aria-hidden="true" {}
                }
            }
        }
    }

    fn head(&self) -> Vec<Markup> {
        vec![html! {
            (PreEscaped(r#"
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
