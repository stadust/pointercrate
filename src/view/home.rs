use super::{url_helper, Page};
use crate::{model::user::User, permissions::Permissions, state::PointercrateState};
use actix_web::{AsyncResponder, HttpRequest, Responder};
use maud::{html, Markup, PreEscaped};
use tokio::prelude::Future;

#[derive(Debug)]
struct Homepage {
    demonlist_team: Vec<User>,
    pointercrate_team: Vec<User>,
}

pub fn handler(req: &HttpRequest<PointercrateState>) -> impl Responder {
    let req_clone = req.clone();

    req.state()
        .get((Permissions::ListAdministrator, Permissions::Administrator))
        .map(move |(demonlist_team, pointercrate_team)| {
            Homepage {
                demonlist_team,
                pointercrate_team,
            }
            .render(&req_clone)
        })
        .responder()
}

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
        vec!["css/home.css"]
    }

    fn body(&self, req: &HttpRequest<PointercrateState>) -> Markup {
        html! {
            div.tabbed.information-banner.left {
                div.tab-display {
                    div.information {
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
                        div.tab-selection.flex.wrap style="padding: 20px 0px; text-align: center"{
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
                    a.big.blue.hover.button.js-scroll-anim data-anim="fade" href = {(url_helper::demon(req, 1))} {
                        "Check it out"(PreEscaped("&nbsp;&nbsp;&nbsp;"))
                        i.fa.fa-arrow-right aria-hidden="true" {}
                    }
                }
            }
            div.center.information-stripe {
                div.flex style="flex-wrap: wrap; align-items: center" {
                    span { "Over 13 000 daily visitors!" }
                    span { "Over 150 ranked demons!" }
                    span { "Over 15 000 records!" }
                }
            }
            div.center.information-banner.right {
                div {
                    a.big.blue.hover.button.js-scroll-anim data-anim="fade" href = "https://github.com/stadust/pointercrate" target = "_blank"{
                        i.fa.fa-github aria-hidden="true" {}
                        (PreEscaped("&nbsp;&nbsp;&nbsp;"))
                        "To the repository"
                    }
                    div.information {
                        h2 { "Now on GitHub "}
                        h3 { "The entirety of the pointercrate codebase can now be found on GitHub"}
                        p{"Found a bug on the website? Want to help with development? Or maybe you just want to find out how everything here works? Head over to the pointercrate GitHub repository!"}
                        p{"Even our custom Geometry Dash API wrapper, GDCF, can be found there!"}
                    }
                }
            }
            div.center.information-stripe {
                div.flex style="flex-wrap: wrap; align-items: center" {
                    span { "Written in Rust!" }
                    span { "Actively Maintained!" }
                    span { "I don't know what to write anymore!" }
                }
            }
            div.tabbed.center.information-banner.left#changelog {
                div.tab-display {
                    div style = "display: flex; flex-flow: column;"{
                        h2 style="text-align: left; margin-top: 0px" {
                            "Changelog"
                        }
                        div.tab-content data-tab-id ="99" {
                            h3 style="text-align: left; font-size: 110%" {
                                "2019-??-??: Rustification!"
                            }
                            p {
                                "The entire website has been rewritten in Rust! Various minor bugs that were noticed while porting over from the old python backend were fixed and performance has greatly improved. Other than that, it's mostly an internal change."
                            }
                            p {
                                "Additionally, I have, yet again, redesigned the home page! Most notably, it has been merged it with the former about page, as both were very under-utilized."
                            }
                            p {
                                "Now onto some more serious topics: As some of you might know, I took up a second undergrad course (mathmatics) in october, meaning my university schedule became much more demanding, leaving me nearly no time to work on pointercrate. Development on discord bots related to pointercrate and the demonlist has already been taken over by GunnerBones, and with pointercrate becoming open source, I'm hoping to find more people will to work on it. In the long run, I'm probably searching for someone who wants to take over pointercrate."
                            }
                        }
                        div.tab-content.tab-content-active data-tab-id ="100" {
                            h3 style="text-align: left" {
                                "2018-04-04: Anniversary Update!"
                            }
                            p {
                                "Its been one year since I rented the pointercrate domain and started hosting the demonlist! Today I'm happy to announce the official pointercrate API, which can be used to programmatically access the demonlist. The documentation can be found"
                                a href = "/documentation/" { " here. " }
                                "Further minor changes include:"
                            }
                            ul {
                                li {
                                    "Internal rework of how list mods are authenticated. They now use the account system."
                                }
                                li {
                                    "The website now embeds nicely into discord!"
                                }
                                li {
                                    "We added a link to the official demonlist discord server, which is an awesome place where I get help with spellchecking"
                                }
                                li {
                                    "There is now a public discord bot that integrates with the demonlist! Find it in the discord server!"
                                }
                                li {
                                    "The API is actually just the first step in something way more awesome hopefully coming \"soon\"... :)"
                                }
                            }
                        }
                        div.tab-content data-tab-id ="101" {
                            h3 style="text-align: left" {
                                "2017-10-29: New design!"
                            }
                            p {
                                "Pointercrate's design has been completely overhauled, to fix various issues we had with the old version:"
                            }
                            ul {
                                li {
                                    "The old homepage was thrown together in 5 minutes and you all knew it"
                                }
                                li {
                                    "Scrollbars were working weirdly or not at all"
                                }
                                li {
                                    "On mobile you couldn't close the demonlist after clicking that weird button in the bottom left corner"
                                }
                                li {
                                    "There was way too much blue"
                                }
                            }
                            p {
                                "Most of these issues arose because the old version was not designed with mobile in mind, and mobile support was 'hacked in' later. The new design uses a mobile-first approach and should be a lot more responsive."
                            }
                        }
                    }
                    div.tab-selection style="padding: 20px 0px; text-align: center"{
                        h3.tab data-tab-id="99" style="padding: 10px; text-align:left" { "2019-??-??" }
                        h3.tab.tab-active data-tab-id="100" style="padding: 10px; text-align:left" { "2018-04-04" }
                        h3.tab data-tab-id="101" style="padding: 10px; text-align: left" { "2017-10-29" }
                    }
                }
            }
            div.center.information-stripe {
                div.flex style="flex-wrap: wrap; align-items: center" {
                    span { "On average updated once a year!" }
                    span { "I redo the homepage every time!" }
                    span { "No new features since 1975!" }
                }
            }
            div.center.information-banner.right {
                div style = "flex-flow: column" {
                    h2#contact {
                        "Contact"
                    }
                    div.flex#about-inner {
                        div style = "flex-basis: 0; padding: 5px" {
                            h3 { "Demonlist Team: "}
                            p {
                                "The demonlist is managed by a large team of players lead by:"
                            }
                            div.flex.wrap style = "padding: 20px" {
                                @for member in &self.demonlist_team {
                                    h4 style="display: inline; margin: 5px" { (member.name) }
                                }
                            }
                            p {
                                "Contant these people for any list related questions/issues"
                            }
                            i {
                                "Twitter: "
                                a href = "https://twitter.com/demonlistgd" {"demonlistgd"}
                            }
                            br ;
                            i {
                                "YouTube: "
                                a href = "https://www.youtube.com/channel/UCqI5feGZEqJRp6VcrP5gVyw" {"Demon List GD"}
                            }
                            br ;
                            i {
                                "Twitch: "
                                a href = "https://twitch.tv/demonlistgd/" {"DemonListGD"}
                            }
                            br ;
                            i {
                                "Discord: "
                                a href = "https://discord.gg/cZcBxQT" {"Demon List Public Server"}
                            }
                        }
                        div style = "flex-basis: 0; padding: 5px" {
                            h3 { "Pointercrate Team: "}
                            p {
                                "Pointercrate as an entity independent from the demonlist is administrated and moderated by the following people:"
                            }
                            div.flex.wrap style = "padding: 20px" {
                                @for member in &self.pointercrate_team {
                                    h4 style="display: inline; margin: 5px" { (member.name) }
                                }
                            }
                            p {
                                "Contact these people for suggestion for pointercrate itself, bug reports or programming related questions"
                            }
                            i {
                                "Twitter: "
                                a href = "https://twitter.com/stadust1971" {"stadust - pointercrate"}
                            }
                            br ;
                            i {
                                "Discord: "
                                a href = "https://discord.gg/sQewUEB" {"Pointercrate Central"}
                            }
                        }
                    }
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
