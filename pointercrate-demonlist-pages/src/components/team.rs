use maud::{html, Markup, Render};
use pointercrate_core::localization::tr;
use pointercrate_user::User;
use unic_langid::LanguageIdentifier;

pub struct Team {
    pub admins: Vec<User>,
    pub moderators: Vec<User>,
    pub helpers: Vec<User>,
    pub lang: &'static LanguageIdentifier,
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
                        (tr(&self.lang, "editors-panel"))
                    }
                }
                p {
                    (tr(&self.lang, "editors-panel.info"))
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
                        (tr(&self.lang, "helpers-panel"))
                    }
                }
                p {
                    (tr(&self.lang, "helpers-panel.info"))
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
