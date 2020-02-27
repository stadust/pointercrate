use super::Page;
use crate::error::PointercrateError;
use maud::{html, Markup, PreEscaped};

#[derive(Debug)]
pub struct ErrorPage<'a> {
    error: &'a PointercrateError,
}

impl ErrorPage<'_> {
    pub fn new(error: &PointercrateError) -> ErrorPage {
        ErrorPage { error }
    }
}

impl Page for ErrorPage<'_> {
    fn title(&self) -> String {
        let status = self.error.status_code();

        format!("{} - {}", status.as_u16(), status.canonical_reason().unwrap_or("What the fuck?"))
    }

    fn description(&self) -> String {
        format!("{}: {}", self.error.error_code(), self.error.to_string())
    }

    fn scripts(&self) -> Vec<&str> {
        vec![]
    }

    fn stylesheets(&self) -> Vec<&str> {
        vec![]
    }

    fn body(&self) -> Markup {
        html! {
            div.m-center.flex.col.cen.no-stretch#error style = "height: calc(100% - 60px)"{
                div.flex.cen style="width: 100%" {
                    svg width="150.98mm" height="84.667mm" version="1.1" viewBox="0 0 150.98 84.667" xmlns="http://www.w3.org/2000/svg"{
                        g transform="translate(-27.214 -41.488)" {
                            text x="102.74387" y="126.15475" fill="#000000" font-family="Norwester" font-size="105.83px" letter-spacing="0px" stroke-width=".26458" text-align="center" text-anchor="middle" word-spacing="0px" style="font-feature-settings:normal;font-variant-caps:normal;font-variant-ligatures:normal;font-variant-numeric:normal;line-height:1.25" xml:space="preserve" {
                                tspan x="98.40271" y="126.15475" font-size="105.83px" letter-spacing="-8.6823px" stroke-width=".26458" text-align="center" text-anchor="middle" style="font-feature-settings:normal;font-variant-caps:normal;font-variant-ligatures:normal;font-variant-numeric:normal" {
                                    ({self.error.error_code() / 100})
                                }
                            }
                        }
                    }
                    div style="max-width: 30%" {
                        h1 style="text-align: right; margin: 0px;" {
                            "Oh No!"
                        }
                        h2 style="text-align: right; margin: 0px" {
                            (self.error.status_code().canonical_reason().unwrap_or("What the fuck?"))
                        }
                    }
                    p.leftlined.pad style = "max-width: 30%" {
                        (self.error.to_string())
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

    fn head(&self) -> Vec<Markup> {
        vec![html! {
            (PreEscaped(r#"
<style>
    @font-face {
        font-family: 'norwester';
        src: url(/static2/norwester.otf);
    }

    *:not(svg){
        position: relative;
    }

    svg {
        position: absolute;
        font-family: norwester;
        max-width: 100%;
        width: 100%;
    }

    svg tspan {
        opacity: 0.5;
        fill: transparent;
        stroke: #AAA;
        stroke-dasharray: 400px;
        stroke-dashoffset: 400px;
        animation-name: draw;
        animation-duration: 5s;
        animation-fill-mode: forwards;
        animation-iteration-count: 1;
        animation-timing-function: linear;
    }

    @keyframes draw {
        50% {
            fill: transparent;
        }
        90% {
            stroke-dashoffset: 0;
        }
        100% {
            fill: #DDD;
        }
    }

    @media (max-width: 767px){
        #error p {
            max-width: unset;
        }
    }
</style>
            "#))
        }]
    }
}
