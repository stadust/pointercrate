use chrono::{DateTime, Datelike, FixedOffset};
use maud::{html, Markup, Render};
use pointercrate_demonlist::demon::TimeShiftedDemon;

pub enum Tardis {
    Activated {
        destination: DateTime<FixedOffset>,
        demons: Vec<TimeShiftedDemon>,
        /// Whether the time selection panel should be visible.
        show_selector: bool,
        /// Whether the "You are currently looking at the demonlist as it was on ..." panel should be visible.
        show_destination: bool,
    },
    Deactivated {
        /// Whether the time selection panel should be visible.
        show_selector: bool,
    },
}

impl Tardis {
    pub fn new(visible: bool) -> Self {
        Tardis::Deactivated { show_selector: visible }
    }

    pub fn activate(&mut self, destination: DateTime<FixedOffset>, demons_then: Vec<TimeShiftedDemon>, show_destination: bool) {
        *self = Tardis::Activated {
            show_selector: self.visible(),
            demons: demons_then,
            destination,
            show_destination,
        };
    }

    pub fn visible(&self) -> bool {
        match self {
            Tardis::Activated {
                show_selector: visible, ..
            } => *visible,
            Tardis::Deactivated { show_selector: visible } => *visible,
        }
    }
}

impl Render for Tardis {
    // maud does not support @if let for complex patterns such as "@if let Tardis::Activated
    // {destination,..} = self". If errors out on the comma.
    #[allow(clippy::single_match)]
    fn render(&self) -> Markup {
        html! {
            @match self {
                Tardis::Activated { destination, show_destination, ..} if *show_destination => {
                    div.panel.fade.blue.flex style="align-items: center;" {
                        span style = "text-align: end"{
                            "You are currently looking at the demonlist how it was on"
                            br;
                            b {
                                @match destination.day() {
                                   1 | 21 | 31 => (destination.format("%A, %B %est %Y at %l:%M:%S%P GMT%Z")),
                                   2 | 22 => (destination.format("%A, %B %end %Y at %l:%M:%S%P GMT%Z")),
                                   3 | 23 => (destination.format("%A, %B %erd %Y at %l:%M:%S%P GMT%Z")),
                                   _ => (destination.format("%A, %B %eth %Y at %l:%M:%S%P GMT%Z"))
                                }
                            }
                        }
                        a.white.button href = "/demonlist/" onclick=r#"document.cookie = "when=""# style = "margin-left: 15px"{ b{"Go to present" }}
                    }
                },
                _ => {}
            }
            section.panel.fade.closable #time-machine  style=(if !self.visible() {"display:none;overflow: initial"} else {"overflow: initial"}) {
                span.plus.cross.hover {}
                form #time-machine-form novalidate = "" {
                    div.underlined {
                        h2 {"Time Machine"}
                    }
                    p {
                        "Enter the date you want to view the demonlist at below. For technical reasons, the earliest possible date is January 4th 2017. Note however that data before August 4th 2017 is only provided on a best-effort basis and not guaranteed to be 100% accurate. Particularly data from before April 4th 2017 contains significant errors!"
                    }
                    div.flex {
                        span.form-input #time-machine-destination data-type = "datetime-local" {
                            h3 {"Destination:"}
                            input name="time-machine-destination" type="datetime-local" min="2017-01-04T00:00" required;
                            p.error {}
                        }
                    }
                    input.button.blue.hover type = "submit" style = "margin: 15px auto 0px;" value="Go!";
                }
            }
        }
    }
}
