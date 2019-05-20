#![allow(unused_variables)]
// currently, all the request parameters are unused, but they will be required in the future

use crate::{
    config::{EXTENDED_LIST_SIZE, LIST_SIZE},
    state::PointercrateState,
};
use actix_web::HttpRequest;
use maud::{html, Markup, PreEscaped, DOCTYPE};

pub mod account;
pub mod demonlist;
pub mod documentation;
pub mod error;
pub mod home;
pub mod login;

// FIXME: we need a better dynamic url generation solution. We cannot use url_for because it breaks
// when running behind a reverse proxy (all URLs it generates are for 127.0.0.1 which is freaking
// useless)
pub const STATIC: &str = "/static2/";

pub trait Page {
    fn title(&self) -> String;
    fn description(&self) -> String;

    fn scripts(&self) -> Vec<&str>;
    fn stylesheets(&self) -> Vec<&str>;

    fn body(&self, req: &HttpRequest<PointercrateState>) -> Markup;

    fn head(&self, req: &HttpRequest<PointercrateState>) -> Vec<Markup>;

    fn render(&self, req: &HttpRequest<PointercrateState>) -> Markup {
        html! {
            (DOCTYPE)
            html lang="en" prefix="og: http://opg.me/ns#" {
                head {
                    title {
                        (self.title())
                    }

                    meta property="og:site_name" content="pointercrate";
                    meta property="og:type" content="website";
                    meta property="og:title" content = (self.title());
                    meta property="og:description" content = (self.description());

                    meta name ="viewport" content="initial-scale=1, maximum-scale=1";
                    meta name="author" content = "stadust, GunnerBones";
                    meta name="keywords" content ="stardust1971,official,geometry,dash,hardest,extreme,insane,demon,list,demonlist,hardest,levels,gmd,gd,stadust,game,top";
                    meta name="description" content = (self.description());
                    meta http-equiv="Content-Type" content = "text/html; charset=utf-8";
                    meta http-equiv="Content-Style-Type" content="text/css";

                    @for markup in self.head(req) {
                        {(markup)}
                    }

                    script src = "https://ajax.googleapis.com/ajax/libs/jquery/3.1.1/jquery.min.js" {}
                    script src = "https://ajax.googleapis.com/ajax/libs/jqueryui/1.12.1/jquery-ui.min.js" {}

                    script src = {(STATIC) "js/nav.v2.js"} {}
                    script src = {(STATIC) "js/misc.v2.js"} {}
                    script src = {(STATIC) "js/ui.v2.js"} {}
                    script src = {(STATIC) "js/tab.js"} {}

                    @for script in self.scripts() {
                        script src = {(STATIC)(script)} {}
                    }

                    link rel = "stylesheet" href = "https://maxcdn.bootstrapcdn.com/font-awesome/4.7.0/css/font-awesome.min.css";
                    link rel = "stylesheet" href = "https://fonts.googleapis.com/css?family=Montserrat|Montserrat:light,bold";

                    link rel = "stylesheet" href = {(STATIC) "css/core/layout.v2.css"};
                    link rel = "stylesheet" href = {(STATIC) "css/core/icon.v2.css"};
                    link rel = "stylesheet" href = {(STATIC) "css/core/nav.v2.css"};
                    link rel = "stylesheet" href = {(STATIC) "css/core/ui.v2.1.css"};
                    link rel = "stylesheet" href = {(STATIC) "css/core/core.v2.css"};
                    link rel = "stylesheet" href = {(STATIC) "css/main.v2.1.css"};
                    link rel = "stylesheet" href = {(STATIC) "css/core/tab.css"};

                    @for sheet in self.stylesheets() {
                        link rel = "stylesheet" href = {(STATIC) (sheet)};
                    }
                }
                body style={"background-image: url(" (STATIC) "images/squares3.png)"}{
                    (nav_bar(req))
                    div {}
                    (self.body(req))
                    (footer(req))
                }
            }
        }
    }
}

pub fn nav_bar(req: &HttpRequest<PointercrateState>) -> Markup {
    html! {
        div.nav.center.collapse.underlined.see-through {
            div.nav-icon {
                a href = "/" {
                    img src = {(STATIC) "images/pointercrate2.png"} style="height:15px";
                }
            }
            div.nav-group-right.nav-group {
                a.nav-item.hover.white href = "/documentation/" {
                    span style ="display:flex; flex-direction:column;" {
                        span style ="font-size: 50%" {"REST API"}
                        span {"Documentation"}
                    }
                }
                a.nav-item.hover.white href = "/demonlist/" title = "Geometry Dash Demonlist" {
                    span style ="display:flex; flex-direction:column;" {
                        span style ="font-size: 50%" {"Geometry Dash"}
                        span {"DEMONLIST"}
                    }
                }
                div.nav-item.collapse-button {
                    div.hamburger.hover {
                        input type="checkbox"{}
                        span{}
                        span{}
                        span{}
                    }
                }
            }
        }
    }
}

pub fn footer(req: &HttpRequest<PointercrateState>) -> Markup {
    let first_extended = *LIST_SIZE + 1;
    let first_legacy = *EXTENDED_LIST_SIZE + 1;

    html! {
        div.footer.center.fade {
            span.overline.pad style="text-align:center" {
                "© Copyright 2017-2019 pointercrate.com"
                br;
                "All rights reserved"
                br;
                "pointercrate.com and the Demonlist are in no way affiliated with RobTopGamesAB ®"
            }
            div.flex.no-stretch {
                div {
                    h2 { "pointercrate:" }
                    a.link.js-scroll {
                        "Back to top"
                    }
                    br ;
                    a.link href = "/#contact" {
                        "Contact"
                    }
                    br ;
                    a.link href = "/documentation/" {
                        "API Documentation"
                    }
                    br ;
                    a.link href = "/login/" {
                        "Staff Area"
                    }
                }
                div {
                    h2 { "Terms of Use:" }
                    "All content on pointercrate.com is provided free of charge. However, you may not redistribute, in any way, any original content found here without the creator's explicit permission."
                }
                div {
                    h2 {
                        "Demonlist:"
                    }
                    a.link href="/demonlist/1/" title = "Hardest demon" {
                        "Current top demon"
                    }
                    br;
                    a.link href = {"/demonlist/" (first_extended) "/"} title="Extended list" {
                        "Extended list"
                    }
                    br;
                    a.link href = {"/demonlist/" (first_legacy) "/"} title="Legacy list" {
                        "Legacy List"
                    }
                }
            }
            div style="display: flex; justify-content: flex-end; align-items: center" {
                i class = "fa fa-twitter fa-2x" {}
                (PreEscaped("&nbsp;&nbsp;Tweet Us:&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;"))
                a href="https://twitter.com/stadust1971" target="_blank" style = "color: #666" {
                    "Developer"
                }
                (PreEscaped("&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;"))
                a href = "https://twitter.com/demonlistgd" target = "_black" style = "color: #666" {
                    "Demonlist Team"
                }
            }
        }
    }
}
