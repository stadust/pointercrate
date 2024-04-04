use crate::{head::HeadLike, PageFragment};
use maud::{html, Markup, PreEscaped};

pub struct ErrorFragment {
    pub status: u16,
    pub reason: String,
    pub message: String,
}

impl From<ErrorFragment> for PageFragment {
    fn from(error: ErrorFragment) -> Self {
        let body = error.body();

        PageFragment::new(format!("{} - {}", error.status, error.reason), error.message)
            .stylesheet("/static/core/css/error.css")
            .body(body)
    }
}

impl ErrorFragment {
    pub fn body(&self) -> Markup {
        html! {
            div.m-center.flex.col.cen.no-stretch #error style = "height: calc(100% - 60px)"{
                div.flex.cen style="width: 100%" {
                    svg width="150.98mm" height="84.667mm" version="1.1" viewBox="0 0 150.98 84.667" xmlns="http://www.w3.org/2000/svg"{
                        g transform="translate(-27.214 -41.488)" {
                            text x="102.74387" y="126.15475" fill="#000000" font-family="Norwester" font-size="105.83px" letter-spacing="0px" stroke-width=".26458" text-align="center" text-anchor="middle" word-spacing="0px" style="font-feature-settings:normal;font-variant-caps:normal;font-variant-ligatures:normal;font-variant-numeric:normal;line-height:1.25" xml:space="preserve" {
                                tspan x="98.40271" y="126.15475" font-size="105.83px" letter-spacing="-8.6823px" stroke-width=".26458" text-align="center" text-anchor="middle" style="font-feature-settings:normal;font-variant-caps:normal;font-variant-ligatures:normal;font-variant-numeric:normal" {
                                    (self.status)
                                }
                            }
                        }
                    }
                    div style="max-width: 30%" {
                        h1 style="text-align: right; margin: 0px;" {
                            "Oh No!"
                        }
                        h2 style="text-align: right; margin: 0px" {
                            (self.reason)
                        }
                    }
                    p.leftlined.pad style = "max-width: 30%" {
                        (self.message)
                    }
                }
                p style="text-align: center; font-size: .7em" {
                    "Believe we've made a mistake in showing you this error?"(PreEscaped("&nbsp;"))
                    a.link href = "/#contact" {
                        "Contact us!"
                    }
                }
            }
        }
    }
}
