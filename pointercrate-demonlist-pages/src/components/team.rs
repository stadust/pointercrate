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
            section.panel.fade.js-scroll-anim#editors data-anim = "fade" {
                div.underlined {
                    h2 {
                        "List Editors:"
                    }
                }
                p {
                    "Contact any of these people if you have problems with the list or want to see a specific thing changed."
                }
                ul style = "line-height: 30px" {
                    @for admin in &self.admins {
                        b {
                            (maybe_link(admin))
                        }
                    }
                    @for moderator in &self.moderators {
                        (maybe_link(moderator))
                    }
                }
                div.underlined {
                    h2 {
                        "List Helpers"
                    }
                }
                p {
                    "Contact these people if you have any questions regarding why a specific record was rejected. Do not needlessly bug them about checking submissions though!"
                }
                ul style = "line-height: 30px" {
                    @for helper in &self.helpers {
                        (maybe_link(helper))
                    }
                }
            }
        }
    }
}
