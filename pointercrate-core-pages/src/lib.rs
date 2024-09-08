use crate::{
    footer::Footer,
    head::{Head, HeadLike},
    navigation::NavigationBar,
};
use maud::{html, Markup, PreEscaped, Render, DOCTYPE};

pub mod config;
pub mod error;
pub mod footer;
pub mod head;
pub mod navigation;
pub mod util;

pub struct PageConfiguration {
    pub footer: Footer,
    pub nav_bar: NavigationBar,
    pub head: Head,
}

impl HeadLike for PageConfiguration {
    fn head_mut(&mut self) -> &mut Head {
        &mut self.head
    }
}

impl PageConfiguration {
    pub fn new(site_name: impl Into<String>, nav_bar: NavigationBar, footer: Footer) -> Self {
        let default_head_html = html! {
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

            meta http-equiv="Content-Type" content = "text/html; charset=utf-8";
            meta http-equiv="Content-Style-Type" content="text/css";
        };

        PageConfiguration {
            footer,
            nav_bar,
            head: Head::new(default_head_html)
                .meta("og:site_name", site_name)
                .meta("og:type", "website")
                .meta("referrer", "no-referrer")
                .meta("viewport", "initial-scale=1, maximum-scale=1")
                .script("https://ajax.googleapis.com/ajax/libs/jquery/3.1.1/jquery.min.js?v=4")
                .script("https://ajax.googleapis.com/ajax/libs/jqueryui/1.12.1/jquery-ui.min.js?v=4")
                .script("/static/core/js/ui.js?v=4")
                .script("/static/core/js/nav.js?v=4")
                .script("/static/core/js/misc.js?v=4")
                .stylesheet("/static/core/css/icon.css")
                .stylesheet("/static/core/css/nav.css")
                .stylesheet("/static/core/css/main.css")
                .stylesheet("/static/core/css/ui.css")
                .stylesheet("/static/core/css/core.css")
                .stylesheet("/static/core/css/fa.all.min.css")
                .stylesheet("https://fonts.googleapis.com/css?family=Montserrat|Montserrat:light,bold"),
        }
    }

    pub fn author(self, author: impl Into<String>) -> Self {
        self.meta("author", author)
    }

    pub fn keywords(self, keywords: impl Into<String>) -> Self {
        self.meta("keywords", keywords)
    }
}

pub struct PageFragment {
    pub head: Head,
    pub body: Markup,
}

impl HeadLike for PageFragment {
    fn head_mut(&mut self) -> &mut Head {
        &mut self.head
    }
}

impl PageFragment {
    pub fn new(title: impl Into<String>, description: impl Into<String>) -> PageFragment {
        let title = title.into();
        let description = description.into();

        PageFragment {
            head: Head::new(html! { title { (title) }}),
            body: html! {},
        }
        .meta("og:title", &title)
        .meta("og:description", &description)
        .meta("description", description)
    }

    pub fn head(mut self, head: Markup) -> Self {
        self.head.other = html! {
            (self.head.other)
            (head)
        };
        self
    }

    pub fn body(mut self, body: Markup) -> Self {
        self.body = body;
        self
    }
}

impl Render for PageFragment {
    fn render(&self) -> Markup {
        html! {
            (DOCTYPE)
            html lang="en" prefix="og: http://opg.me/ns#" {
                head {
                    (self.head)
                }
                body style="z-index:-10" {
                    (self.body)
                }
            }
        }
    }
}
