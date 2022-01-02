use maud::{html, Markup, PreEscaped};

use pointercrate_demonlist::{config, demon::Demon};

pub mod account;
pub mod components;
pub mod demon_page;
pub mod overview;
pub mod statsviewer;

struct ListSection {
    name: &'static str,
    description: &'static str,
    id: &'static str,
    numbered: bool,
}

static MAIN_SECTION: ListSection = ListSection {
    name: "Main List",
    description: "The main section of the Demonlist. These demons are the hardest rated levels in the game. Records are accepted above a \
                  given threshold and award a large amount of points!",
    id: "mainlist",
    numbered: true,
};

static EXTENDED_SECTION: ListSection = ListSection {
    name: "Extended List",
    description: "These are demons that dont qualify for the main section of the list, but are still of high relevance. Only 100% records \
                  are accepted for these demons! Note that non-100% that were submitted/approved before a demon fell off the main list \
                  will be retained",
    id: "extended",
    numbered: true,
};

static LEGACY_SECTION: ListSection = ListSection {
    name: "Legacy List",
    description: "These are demons that used to be on the list, but got pushed off as new demons were added. They are here for nostalgic \
                  reasons. This list is in no order whatsoever and will not be maintained any longer at all. This means no new records \
                  will be added for these demons.",
    id: "legacy",
    numbered: false,
};

fn dropdowns(all_demons: &[&Demon], current: Option<&Demon>) -> Markup {
    let (main, extended, legacy) = if all_demons.len() < config::list_size() as usize {
        (&all_demons[..], Default::default(), Default::default())
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
        nav.flex.wrap.m-center.fade#lists style="text-align: center;" {
            // The drop down for the main list:
            (dropdown(&MAIN_SECTION, main, current))
            // The drop down for the extended list:
            (dropdown(&EXTENDED_SECTION, extended, current))
            // The drop down for the legacy list:
            (dropdown(&LEGACY_SECTION, legacy, current))
        }
    }
}

fn dropdown(section: &ListSection, demons: &[&Demon], current: Option<&Demon>) -> Markup {
    let format = |demon: &Demon| -> Markup {
        html! {
            a href = {"/demonlist/permalink/" (demon.base.id) "/"} {
                @if section.numbered {
                    {"#" (demon.base.position) " - " (demon.base.name)}
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

            div.see-through.fade.dropdown#(section.id) {
                div.search.js-search.seperated style = "margin: 10px" {
                    input placeholder = "Filter..." type = "text" {}
                }
                p style = "margin: 10px" {
                    (section.description)
                }
                ul.flex.wrap.space {
                    @for demon in demons {
                        @match current {
                            Some(current) if current.base.position == demon.base.position =>
                                li.hover.white.active title={"#" (demon.base.position) " - " (demon.base.name)} {
                                    (format(demon))
                                },
                            _ =>
                                li.hover.white title={"#" (demon.base.position) " - " (demon.base.name)} {
                                    (format(demon))
                                }
                        }
                    }
                }
            }
        }
    }
}

fn sidebar_ad() -> Markup {
    html! {
        section.panel.fade.js-scroll-anim data-anim = "fade" style = "order: 1; padding: 0px; border: 0" {
            (PreEscaped(format!(r#"
            <script async src="https://pagead2.googlesyndication.com/pagead/js/adsbygoogle.js?client={0}"
     crossorigin="anonymous"></script>
<!-- Demonlist Sidebar Ad -->
<ins class="adsbygoogle"
     style="display:block"
     data-ad-client="{0}"
     data-ad-slot="2559641548"
     data-ad-format="auto"
     data-full-width-responsive="true"></ins>
<script>
     (adsbygoogle = window.adsbygoogle || []).push({{}});
</script>
            "#, pointercrate_core_pages::config::adsense_publisher_id())))
        }
    }
}

fn besides_sidebar_ad() -> Markup {
    html! {
        div#outofboundsad style="margin-left: calc(45% + 1072px/2);position: fixed;padding-left: 15px;padding-top: 15px; max-width: 200px" {
            (PreEscaped(format!(r#"
                <script async src="https://pagead2.googlesyndication.com/pagead/js/adsbygoogle.js?client={0}"
     crossorigin="anonymous"></script>
<!-- Demonlist Sidebar Ad #2 -->
<ins class="adsbygoogle"
     style="display:block"
     data-ad-client="{0}"
     data-ad-slot="3380750697"
     data-ad-format="auto"
     data-full-width-responsive="true"></ins>
<script>
     (adsbygoogle = window.adsbygoogle || []).push({{}});
</script>
            "#, pointercrate_core_pages::config::adsense_publisher_id())))
        }
    }
}

fn rules_panel() -> Markup {
    html! {
        section#rules.panel.fade.js-scroll-anim data-anim = "fade" {
            h2.underlined.pad.clickable {
                "Guidelines:"
            }
            p {
                "All demonlist operations are carried out in accordance to our guidelines. Be sure to check them before submitting a record to ensure a flawless experience!"
            }
            a.blue.hover.button href = "/guidelines/" {
                "Read the guidelines!"
            }
        }
    }
}

fn discord_panel() -> Markup {
    html! {
        section.panel.fade.js-scroll-anim#discord data-anim = "fade" {
            iframe.js-delay-attr style = "width: 100%; height: 400px;" allowtransparency="true" frameborder = "0" data-attr = "src" data-attr-value = "https://discordapp.com/widget?id=395654171422097420&theme=light" {}
            p {
                "Join the official Demonlist discord server, where you can get in touch with the demonlist team!"
            }
        }
    }
}
