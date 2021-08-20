use crate::view::{footer::Footer, navigation::NavigationBar};
use maud::{html, Markup, PreEscaped};

pub mod error;
pub mod footer;
pub mod navigation;

pub struct PageConfiguration {
    pub footer: Footer,
    pub nav_bar: NavigationBar,
    pub default_scripts: Vec<String>,
    pub default_stylesheets: Vec<String>,
}

pub trait PageFragment {
    fn additional_scripts(&self) -> Vec<String>;
    fn additional_stylesheets(&self) -> Vec<String>;

    fn fragment(&self) -> Markup;
}

impl PageConfiguration {
    pub fn render_fragment<F: PageFragment>(&self, fragment: &F) -> Markup {
        html! {
            head {

            }
            body {
                (self.nav_bar)
                (fragment.fragment())
                (self.footer)
            }
        }
    }
}
