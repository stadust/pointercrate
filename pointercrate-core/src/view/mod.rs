use crate::view::{footer::Footer, navigation::NavigationBar};
use maud::{html, Markup};

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
                @for script in &self.default_scripts {
                    script src = (script);
                }
                @for script in fragment.additional_scripts() {
                    script src = (script);
                }

                @for stylesheet in &self.default_stylesheets {
                    link rel = "stylesheet" href = (stylesheet);
                }
                @for stylesheet in fragment.additional_stylesheets() {
                    link rel = "stylesheet" href = (stylesheet);
                }
            }
            body {
                (self.nav_bar)
                (fragment.fragment())
                (self.footer)
            }
        }
    }
}
