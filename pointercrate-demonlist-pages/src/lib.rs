use maud::{html, Markup};

use pointercrate_core::localization::tr;
use pointercrate_demonlist::{config, demon::Demon, list::List};

pub mod account;
pub mod components;
pub mod demon_page;
pub mod overview;
pub mod statsviewer;

struct ListSection {
    name: String,
    description: String,
    id: &'static str,
    numbered: bool,
}

fn dropdowns(all_demons: &[&Demon], list: &List, current: Option<&Demon>) -> Markup {
    let (main, extended, legacy) = if all_demons.len() < config::list_size() as usize {
        (all_demons, Default::default(), Default::default())
    } else {
        let (extended, legacy) = if all_demons.len() < config::extended_list_size() as usize {
            (&all_demons[config::list_size() as usize..], Default::default())
        } else {
            (
                &all_demons[config::list_size() as usize..config::extended_list_size() as usize],
                &all_demons[config::extended_list_size() as usize..],
            )
        };

        (&all_demons[..config::list_size() as usize], extended, legacy)
    };

    html! {
        nav.flex.wrap.m-center.fade #lists style="text-align: center;" {
            // The drop down for the main list:
            (dropdown(&ListSection { name: tr("main-list"), description: tr("main-list.info"), id: "mainlist", numbered: true }, main, list, current))
            // The drop down for the extended list:
            (dropdown(&ListSection { name: tr("extended-list"), description: tr("extended-list.info"), id: "extended", numbered: true }, extended, list, current))
            // The drop down for the legacy list:
            (dropdown(&ListSection { name: tr("legacy-list"), description: tr("legacy-list.info"), id: "legacy", numbered: false }, legacy, list, current))
        }
    }
}

fn dropdown(section: &ListSection, demons: &[&Demon], list: &List, current: Option<&Demon>) -> Markup {
    let format = |demon: &Demon| -> Markup {
        html! {
            a href = (format!("/{}/permalink/{}/", list.as_str(), demon.base.id)) {
                @if section.numbered {
                    {"#" (demon.base.position(list).unwrap()) " - " (demon.base.name)}
                    br ;
                    i {
                        (demon.publisher.name)
                    }
                }
                @else {
                    {(demon.base.name)}
                    br ;
                    i {
                        (demon.publisher.name)
                    }
                }
            }
        }
    };

    html! {
        div {
            div.button.white.hover.no-shadow.js-toggle data-toggle-group="0" onclick={"javascript:void(DropDown.toggleDropDown('" (section.id) "'))"} {
                (section.name)
            }

            div.see-through.fade.dropdown #(section.id) {
                div.search.js-search.seperated style = "margin: 10px" {
                    input placeholder = "Filter..." type = "text" {}
                }
                p style = "margin: 10px" {
                    (section.description)
                }
                ul.flex.wrap.space {
                    @for demon in demons {
                        @match current {
                            Some(current) if current.base.position(list) == demon.base.position(list) =>
                                li.hover.white.active title={"#" (demon.base.position(list).unwrap()) " - " (demon.base.name)} {
                                    (format(demon))
                                },
                            _ =>
                                li.hover.white title={"#" (demon.base.position(list).unwrap()) " - " (demon.base.name)} {
                                    (format(demon))
                                }
                        }
                    }
                }
            }
        }
    }
}

fn rules_panel() -> Markup {
    html! {
        section #rules.panel.fade.js-scroll-anim data-anim = "fade" {
            h2.underlined.pad.clickable {
                (tr("guidelines-panel"))
            }
            p {
                (tr("guidelines-panel.info"))
            }
            a.blue.hover.button href = "/guidelines/" {
                (tr("guidelines-panel.button"))
            }
        }
    }
}

fn discord_panel() -> Markup {
    html! {
        section.panel.fade.js-scroll-anim #discord data-anim = "fade" {
            iframe.js-delay-attr style = "width: 100%; height: 400px;" allowtransparency="true" frameborder = "0" data-attr = "src" data-attr-value = "https://discordapp.com/widget?id=395654171422097420&theme=light" {}
            p {
                (tr("discord-panel-info"))
            }
        }
    }
}
