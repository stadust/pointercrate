use maud::{html, Markup, Render};
use pointercrate_core::localization::LocaleConfiguration;
use unic_langid::subtags::Region;

pub struct TopLevelNavigationBarItem {
    item: NavigationBarItem,
    sub_levels: Vec<NavigationBarItem>,
}

impl TopLevelNavigationBarItem {
    pub fn new(link: Option<&'static str>, content: Markup) -> Self {
        TopLevelNavigationBarItem {
            item: NavigationBarItem { link, content },
            sub_levels: vec![],
        }
    }

    pub fn with_sub_item(mut self, link: Option<&'static str>, content: Markup) -> Self {
        self.sub_levels.push(NavigationBarItem { link, content });
        self
    }
}

struct NavigationBarItem {
    content: Markup,
    link: Option<&'static str>,
}

pub struct NavigationBar {
    logo_path: &'static str,
    items: Vec<TopLevelNavigationBarItem>,
}

impl NavigationBar {
    pub fn new(logo_path: &'static str) -> Self {
        NavigationBar { logo_path, items: vec![] }
    }

    pub fn with_item(mut self, item: TopLevelNavigationBarItem) -> Self {
        self.items.push(item);
        self
    }
}

struct NavGroup<T> {
    inner: T,
    id: Option<&'static str>,
    nohide: bool,
}

impl<T> NavGroup<T> {
    pub fn new(inner: T) -> Self {
        Self {
            inner,
            id: None,
            nohide: false,
        }
    }
}

impl<T: Render> Render for NavGroup<T> {
    fn render(&self) -> Markup {
        html! {
            div.nav-group.nav-nohide[self.nohide] id = [self.id] {
                (self.inner)
            }
        }
    }
}

impl Render for TopLevelNavigationBarItem {
    fn render(&self) -> Markup {
        html! {
            a.nav-item.hover.white href = [self.item.link] {
                (self.item.content)
                @if !self.sub_levels.is_empty() {
                    i.fas.fa-sort-down style = "height: 50%; padding-left: 5px" {}
                }
            }
            @if !self.sub_levels.is_empty() {
                ul.nav-hover-dropdown {
                    @for sub_item in &self.sub_levels {
                        li {
                            a.white.hover href = [sub_item.link] { (sub_item.content)}
                        }
                    }
                }
            }
        }
    }
}

impl Render for NavigationBar {
    fn render(&self) -> Markup {
        html! {
            header {
                nav.center.collapse.underlined.see-through {
                    div.nav-icon.nav-nohide style = "margin-right: auto" {
                        a href = "/" aria-label = "Go to homepage" {
                            img src = (self.logo_path) style="height:15px" alt="Logo";
                        }
                    }
                    @for item in &self.items {
                        (NavGroup::new(item))
                    }
                    @if let Some(locales_dropdown) = locale_selection_dropdown() {
                        (locales_dropdown)
                    }
                    div.nav-item.collapse-button.nav-nohide {
                        div.hamburger.hover {
                            input type="checkbox"{}
                            span{}
                            span{}
                            span{}
                        }
                    }
                    div.nav-drop-down {
                        @for item in &self.items {
                            (item)
                        }
                    }
                }
                div {} // artificial spacing
            }
        }
    }
}

fn flag(region: Option<Region>) -> Markup {
    html! [
        @if let Some(region) = region {
            span.flag-icon style = (format!(r#"background-image: url("/static/demonlist/images/flags/{}.svg");"#, region.as_str().to_ascii_lowercase())) {}
        }
    ]
}

fn locale_selection_dropdown() -> Option<NavGroup<TopLevelNavigationBarItem>> {
    let config = LocaleConfiguration::get();
    let locales = config.locales();

    let active_locale = config.active_locale();

    if locales.len() < 2 {
        return None;
    }

    let mut dropdown = TopLevelNavigationBarItem::new(
        None,
        html! {
            span.flex {
                (flag(active_locale.region))
                span #active-language style = "margin-left: 8px" { (active_locale.language.as_str().to_uppercase()) }
            }
        },
    );

    for locale in locales {
        if locale == active_locale {
            // this locale is currently selected, don't add it to the dropdown
            continue;
        }

        dropdown = dropdown.with_sub_item(
            None,
            html! {
                span data-lang = (locale) {
                    (flag(locale.region))
                    span style = "margin-left: 10px" { (locale.language.as_str().to_uppercase()) }
                }
            },
        );
    }

    Some(NavGroup {
        inner: dropdown,
        id: Some("language-selector"),
        nohide: true,
    })
}
