use maud::{html, Markup, Render};

#[derive(Debug, Clone)]
pub struct Head {
    scripts: Vec<Script>,
    stylesheets: Vec<String>,
    meta_tags: Vec<Meta>,

    pub(crate) other: Markup,
}

impl Head {
    pub fn new(other: Markup) -> Head {
        Head {
            scripts: vec![],
            stylesheets: vec![],
            meta_tags: vec![],
            other,
        }
    }
}

impl Render for Head {
    fn render(&self) -> Markup {
        html! {
            @for meta in &self.meta_tags {
                meta name=(meta.name) content=(meta.content);
            }

            @for script in &self.scripts {
                (script)
            }

            @for stylesheet in &self.stylesheets {
                link rel = "stylesheet" href = (stylesheet);
            }

            (self.other)
        }
    }
}

pub trait HeadLike: Sized {
    fn head_mut(&mut self) -> &mut Head;

    fn with_stylesheet(mut self, url: String) -> Self {
        self.head_mut().stylesheets.push(url);
        self
    }

    fn with_script(mut self, script: Script) -> Self {
        self.head_mut().scripts.push(script);
        self
    }

    fn with_meta(mut self, meta: Meta) -> Self {
        self.head_mut().meta_tags.push(meta);
        self
    }

    fn meta(self, name: impl Into<String>, content: impl Into<String>) -> Self {
        self.with_meta(Meta::new(name, content))
    }

    fn script(self, src: impl Into<String>) -> Self {
        self.with_script(Script::new(src))
    }

    fn module(self, module: impl Into<String>) -> Self {
        self.with_script(Script::module(module))
    }

    fn stylesheet(self, sheet: impl Into<String>) -> Self {
        self.with_stylesheet(sheet.into())
    }
}

impl HeadLike for Head {
    fn head_mut(&mut self) -> &mut Head {
        self
    }
}

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

#[derive(Debug, Clone)]
pub struct Meta {
    name: String,
    content: String,
}

impl Meta {
    pub fn new(name: impl Into<String>, content: impl Into<String>) -> Meta {
        Meta {
            name: name.into(),
            content: content.into(),
        }
    }
}

impl Render for &Meta {
    fn render(&self) -> Markup {
        html! {
            meta name=(self.name) property=(self.name) content=(self.content);
        }
    }
}

impl Render for Script {
    fn render(&self) -> Markup {
        (&self).render()
    }
}
