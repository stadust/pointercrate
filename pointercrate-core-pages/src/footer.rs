use maud::{html, Markup, PreEscaped, Render};

pub struct Footer {
    copyright_notice: Markup,
    columns: Vec<FooterColumn>,
    twitter_links: Vec<Link>,
}

impl Footer {
    pub fn new(copyright_notice: Markup) -> Self {
        Footer {
            copyright_notice,
            columns: vec![],
            twitter_links: vec![],
        }
    }

    pub fn with_column(mut self, column: FooterColumn) -> Self {
        self.columns.push(column);
        self
    }

    pub fn with_link(mut self, href: &'static str, text: &'static str) -> Self {
        self.twitter_links.push(Link::new(href, text));
        self
    }
}

pub enum FooterColumn {
    LinkList { heading: &'static str, links: Vec<Link> },
    Arbitrary { heading: &'static str, content: Markup },
}

pub struct Link {
    href: String,
    text: String,
}

impl Link {
    pub fn new<S: Into<String>, T: Into<String>>(href: S, text: T) -> Self {
        Link {
            href: href.into(),
            text: text.into(),
        }
    }
}

impl Render for &Link {
    fn render(&self) -> Markup {
        html! {
            a.link href = (self.href) {
                (self.text)
            }
        }
    }
}

impl Render for &FooterColumn {
    fn render(&self) -> Markup {
        match self {
            FooterColumn::LinkList { heading, links } =>
                html! {
                    nav {
                        h2 {(heading)}
                        @for link in links {
                            (link)
                            br;
                        }
                    }
                },
            FooterColumn::Arbitrary { heading, content } =>
                html! {
                    div {
                        h2 {(heading)}
                        (*content)
                    }
                },
        }
    }
}

impl Render for Footer {
    fn render(&self) -> Markup {
        html! {
            footer.center {
                span.overline.pad style="text-align:center" {
                    (self.copyright_notice)
                }
                div.flex.no-stretch {
                    @for column in &self.columns {
                        (column)
                    }
                }
                div style="display: flex; justify-content: flex-end; align-items: center" {
                    i class = "fab fa-twitter fa-2x" {}
                    (PreEscaped("&nbsp;&nbsp;Tweet Us:"))
                    @for link in &self.twitter_links {
                        (PreEscaped("&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;"))
                        a href=(link.href) target="_blank" style = "color:#666" {(link.text)}
                    }
                }
            }
        }
    }
}
