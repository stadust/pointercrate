use crate::{footer::Footer, navigation::NavigationBar};
use maud::{html, Markup, PreEscaped, Render, DOCTYPE};

pub mod config;
pub mod error;
pub mod footer;
pub mod navigation;
pub mod util;

#[derive(Debug, Clone)]
pub struct Script {
    src: String,
    module: bool,
}

impl Script {
    pub fn new<S: Into<String>>(src: S) -> Self {
        Script {
            src: src.into(),
            module: false,
        }
    }

    pub fn module<S: Into<String>>(src: S) -> Self {
        Script {
            src: src.into(),
            module: true,
        }
    }
}

impl Render for &Script {
    fn render(&self) -> Markup {
        html! {
            @if self.module {
                script src = (self.src) type = "module" {}
            }
            @else {
                script src = (self.src) {};
            }
        }
    }
}

impl Render for Script {
    fn render(&self) -> Markup {
        (&self).render()
    }
}

pub struct PageMetadata {
    pub site_name: String,
    pub site_author: String,
    pub keywords: String,
}

pub struct PageConfiguration {
    pub footer: Footer,
    pub nav_bar: NavigationBar,
    pub default_scripts: Vec<Script>,
    pub default_stylesheets: Vec<String>,

    pub meta: PageMetadata,
}

pub trait PageFragment {
    fn title(&self) -> String;
    fn description(&self) -> String;

    fn additional_scripts(&self) -> Vec<Script>;
    fn additional_stylesheets(&self) -> Vec<String>;

    fn head_fragment(&self) -> Markup;
    fn body_fragment(&self) -> Markup;
}

impl PageConfiguration {
    pub fn render_fragment<F: PageFragment>(&self, fragment: &F) -> Markup {
        html! {
            (DOCTYPE)
            html lang="en" prefix="og: http://opg.me/ns#" {
                head {
                    title {
                        (fragment.title())
                    }

                    (PreEscaped(format!(r#"<script async src="https://pagead2.googlesyndication.com/pagead/js/adsbygoogle.js?client={}" crossorigin="anonymous"></script>"#, config::adsense_publisher_id())))

                    (PreEscaped(format!(r#"
                    <!-- Global site tag (gtag.js) - Google Analytics -->
                    <script async src="https://www.googletagmanager.com/gtag/js?id=G-2SGJ4S0TQM"></script>
                    <script>
                      window.dataLayer = window.dataLayer || [];
                      function gtag(){{dataLayer.push(arguments);}}
                      gtag('js', new Date());
                    
                      gtag('config', '{}');
                    </script>
                    "#, config::google_analytics_tag())));

                    meta property="og:site_name" content=(self.meta.site_name);
                    meta property="og:type" content="website";
                    meta property="og:title" content = (fragment.title());
                    meta property="og:description" content = (fragment.description());

                    meta name="referrer" content = "no-referrer";
                    meta name="viewport" content = "initial-scale=1, maximum-scale=1";
                    meta name="author"   content = (self.meta.site_author);
                    meta name="keywords" content = (self.meta.keywords);
                    meta name="description" content = (fragment.description());

                    meta http-equiv="Content-Type" content = "text/html; charset=utf-8";
                    meta http-equiv="Content-Style-Type" content="text/css";

                    @for script in &self.default_scripts {
                        (script)
                    }
                    @for script in fragment.additional_scripts() {
                        (script)
                    }

                    @for stylesheet in &self.default_stylesheets {
                        link rel = "stylesheet" href = (stylesheet);
                    }
                    @for stylesheet in fragment.additional_stylesheets() {
                        link rel = "stylesheet" href = (stylesheet);
                    }

                    (fragment.head_fragment())
                }
                body style="z-index:-10" {
                    // target this element to get background image
                    div style={"width: 100%;height: 100%;position: fixed;top: 0;left: 0;background-size: cover;background-repeat: repeat-y;pointer-events: none; z-index:-1"} {}

                    (self.nav_bar)
                    (fragment.body_fragment())
                    (self.footer)
                }
            }
        }
    }
}
