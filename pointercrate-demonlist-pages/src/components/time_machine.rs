use chrono::{DateTime, Datelike, FixedOffset, TimeZone, Utc};
use maud::{html, Markup, PreEscaped, Render};
use pointercrate_core_pages::util::{filtered_paginator, simple_dropdown};

pub struct TimeMachine {
    initially_visible: bool,
    when: Option<DateTime<FixedOffset>>,
}

impl TimeMachine {
    pub fn new(visible: bool) -> Self {
        TimeMachine {
            initially_visible: visible,
            when: None,
        }
    }

    pub fn at(mut self, when: DateTime<FixedOffset>) -> Self {
        self.when = Some(when);
        self
    }
}

impl Render for TimeMachine {
    fn render(&self) -> Markup {
        let current_year = FixedOffset::east(3600 * 23 + 3599)
            .from_utc_datetime(&Utc::now().naive_utc())
            .year();

        let months = [
            "January",
            "February",
            "March",
            "April",
            "May",
            "June",
            "July",
            "August",
            "September",
            "October",
            "November",
            "December",
        ];

        html! {
            @if let Some(when) = self.when {
                div.panel.fade.blue.flex style="align-items: center;" {
                     span style = "text-align: end"{
                        "You are currently looking at the demonlist how it was on"
                         br;
                         b {
                             @match when.day() {
                                1 | 21 | 31 => (when.format("%A, %B %est %Y at %l:%M:%S%P GMT%Z")),
                                2 | 22 => (when.format("%A, %B %end %Y at %l:%M:%S%P GMT%Z")),
                                _ => (when.format("%A, %B %eth %Y at %l:%M:%S%P GMT%Z"))
                             }
                         }
                     }
                     a.white.button href = "/demonlist/" onclick=r#"document.cookie = "when=""# style = "margin-left: 15px"{ b{"Go to present" }}
                }
            }
            section.panel.fade.closable#time-machine  style=(if !self.initially_visible {"display:none;overflow: initial"} else {"overflow: initial"}) {
                span.plus.cross.hover {}
                form#time-machine-form novalidate = "" {
                    div.underlined {
                        h2 {"Time Machine"}
                    }
                    p {
                        "Enter the date you want to view the demonlist at below. For technical reasons, the earliest possible date is January 4th 2017. Note however that data before August 4th 2017 is only provided on a best-effort basis and not guaranteed to be 100% accurate. Particularly data from before April 4th 2017 contains significant errors!"
                    }
                    div.flex {
                        span.form-input data-type = "dropdown" style = "max-width:33%" {
                            h3 {"Year:"}
                            (simple_dropdown("time-machine-year", None, 2017..=current_year))
                            p.error {}
                        }
                        span.form-input data-type = "dropdown" style = "max-width:33%"  {
                            h3 {"Month:"}
                            (simple_dropdown("time-machine-month", None, months.iter()))
                            p.error {}
                        }
                        span.form-input data-type = "dropdown" style = "max-width:33%"  {
                            h3 {"Day:"}
                            (simple_dropdown("time-machine-day", None, 1..=31))
                            p.error {}
                        }
                    }
                    div.flex {
                        span.form-input data-type = "dropdown" style = "max-width:33%" {
                            h3 {"Hour:"}
                            (simple_dropdown("time-machine-hour", Some(0), 0..24))
                            p.error {}
                        }
                        span.form-input data-type = "dropdown" style = "max-width:33%"  {
                            h3 {"Minute:"}
                            (simple_dropdown("time-machine-minute", Some(0), 0..=59))
                            p.error {}
                        }
                        span.form-input data-type = "dropdown" style = "max-width:33%"  {
                            h3 {"Second:"}
                            (simple_dropdown("time-machine-second", Some(0), 0..=59))
                            p.error {}
                        }
                    }
                    input.button.blue.hover type = "submit" style = "margin: 15px auto 0px;" value="Go!";
                }
            }
        }
    }
}
