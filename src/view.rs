use crate::{
    config::{EXTENDED_LIST_SIZE, LIST_SIZE},
    state::PointercrateState,
};
use actix_web::HttpRequest;
use maud::{html, Markup, PreEscaped, DOCTYPE};

pub mod demonlist;
pub mod documentation;
pub mod error;
pub mod home;

// TODO: maybe do this with `url_for()`

pub const STATIC: &str = "/static/";

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

                    meta property="og:site_name" content="pointercrate" {}
                    meta property="og:type" content="website" {}
                    meta property="og:title" content = (self.title()) {}
                    meta property="og:description" content = (self.description()) {}

                    meta name ="viewport" content="initial-scale=1, maximum-scale=1" {}
                    meta name="author" content = "stadust, GunnerBones" {}
                    meta name="keywords" content ="stardust1971,official,geometry,dash,hardest,extreme,insane,demon,list,demonlist,hardest,levels,gmd,gd,stadust,official,game,top" {}
                    meta name="description" content = (self.description()) {}
                    meta http-equiv="Content-Type" content = "text/html; charset=utf-8" {}
                    meta http-equiv="Content-Style-Type" content="text/css" {}

                    @for markup in self.head(req) {
                        {(markup)}
                    }

                    script src = "https://ajax.googleapis.com/ajax/libs/jquery/3.1.1/jquery.min.js" {}
                    script src = "https://ajax.googleapis.com/ajax/libs/jqueryui/1.12.1/jquery-ui.min.js" {}

                    script src = {(STATIC) "js/nav.v2.js"} {}
                    script src = {(STATIC) "js/misc.v2.js"} {}
                    script src = {(STATIC) "js/ui.v2.js"} {}

                    @for script in self.scripts() {
                        script src = {(STATIC)(script)} {}
                    }

                    link rel = "stylesheet" href = "https://maxcdn.bootstrapcdn.com/font-awesome/4.7.0/css/font-awesome.min.css" {}
                    link rel = "stylesheet" href = "https://fonts.googleapis.com/css?family=Montserrat|Montserrat:light,bold" {}

                    link rel = "stylesheet" href = {(STATIC) "css/core/layout.v2.css"} {}
                    link rel = "stylesheet" href = {(STATIC) "css/core/icon.v2.css"} {}
                    link rel = "stylesheet" href = {(STATIC) "css/core/nav.v2.css"} {}
                    link rel = "stylesheet" href = {(STATIC) "css/core/ui.v2.1.css"} {}
                    link rel = "stylesheet" href = {(STATIC) "css/core/core.v2.css"} {}
                    link rel = "stylesheet" href = {(STATIC) "css/main.v2.1.css"} {}

                    @for sheet in self.stylesheets() {
                        link rel = "stylesheet" href = {(STATIC) (sheet)} {}
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
                a href = (url_helper::url(req, "home")) {
                    img src = {(STATIC) "images/pointercrate2.png"} style="height:15px";
                }
            }
            div.nav-group-right.nav-group {
                a.nav-item.hover.white href = {(url_helper::url(req, "documentation"))} {
                     pan style ="display:flex; flex-direction:column;" {
                        span style ="font-size: 50%" {"REST API"}
                        span {"Documentation"}
                    }
                }
                a.nav-item.hover.white href = {(url_helper::demon(req, 1))} title = "Geometry Dash Demonlist" {
                    span style ="display:flex; flex-direction:column;" {
                        span style ="font-size: 50%" {"Geometry Dash"}
                        span {"DEMONLIST"}
                    }
                }
                a.nav-item.hover.white href = {(url_helper::url(req, "about"))} title="About" {
                    i.fa.fa-info-circle{} (PreEscaped("&nbsp;")) "ABOUT"
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
    let first_extended = &*LIST_SIZE + 1;
    let first_legacy = &*EXTENDED_LIST_SIZE + 1;

    html! {
        div.footer.center.fade {
            span.overline.pad style="text-align:center" {
                "© Copyright 2017-2018 pointercrate.com"
                br;
                "All rights reserved"
                br;
                "pointercrate.com and the Demonlist are in no way affiliated with RobTopGamesAB ®"
            }
            div.flex.no-stretch {
                div {
                    h2 {
                        "pointercrate"
                    }
                    a.link href={ (url_helper::demon(req, 1)) } title = "Hardest demon" {
                        "Current top demon"
                    }
                    br;
                    a.link href = {(url_helper::demon(req, first_extended))} title="Extended list" {
                        "Extended list"
                    }
                    br;
                    a.link href = {(url_helper::demon(req, first_legacy))} title="Legacy list" {
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

pub mod url_helper {
    use crate::state::PointercrateState;
    use actix_web::HttpRequest;
    use log::error;
    use url::Url;

    pub fn demon(req: &HttpRequest<PointercrateState>, position: i16) -> Url {
        url_with(req, "demonlist", &[position])
    }

    pub fn url(req: &HttpRequest<PointercrateState>, name: &str) -> Url {
        req.url_for_static(name)
            .map_err(|_| error!("Internal Error: Attempt to retrieve url for non-existing page"))
            .unwrap()
    }

    pub fn url_with<U, I>(req: &HttpRequest<PointercrateState>, name: &str, params: U) -> Url
    where
        U: IntoIterator<Item = I>,
        I: ToString,
    {
        req.url_for(name, params.into_iter().map(|i| i.to_string()))
            .map_err(|_| error!("Internal Error: Attempt to retrieve url for non-existing page"))
            .unwrap()
    }
}
