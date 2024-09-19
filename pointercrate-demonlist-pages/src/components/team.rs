use maud::{html, Markup, Render};
use pointercrate_user::User;

pub struct Team {
    pub admins: Vec<User>,
    pub moderators: Vec<User>,
    pub helpers: Vec<User>,
}

impl Render for Team {
    fn render(&self) -> Markup {
        let maybe_link = |user: &User| -> Markup {
            html! {
                li {
                    @match user.youtube_channel {
                        Some(ref channel) => a target = "_blank" href = (channel) {
                            (user.name())
                        },
                        None => (user.name())
                    }
                }
            }
        };

        html! {
            section.panel.fade.js-scroll-anim #editors data-anim = "fade" {
                div.underlined {
                    h2 {
                        "List Team"
                    }
                }
                ul style = "line-height: 30px" {
                    @for admin in &self.admins {
                        b {
                            (maybe_link(admin))
                        }
                    }
                    @for moderator in &self.moderators {
                        b {
                            (maybe_link(moderator))
                        }
                    }
                    @for helper in &self.helpers {
                        (maybe_link(helper))
                    }
                }
            }
        }
    }
}
