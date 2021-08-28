use maud::{html, Markup, Render};

pub struct TopLevelNavigationBarItem {
    item: NavigationBarItem,
    sub_levels: Vec<NavigationBarItem>,
}

impl TopLevelNavigationBarItem {
    pub fn new(link: &'static str, content: Markup) -> Self {
        TopLevelNavigationBarItem {
            item: NavigationBarItem { link, content },
            sub_levels: vec![],
        }
    }

    pub fn with_sub_item(mut self, link: &'static str, content: Markup) -> Self {
        self.sub_levels.push(NavigationBarItem { link, content });
        self
    }
}

struct NavigationBarItem {
    content: Markup,
    link: &'static str,
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

impl Render for &TopLevelNavigationBarItem {
    fn render(&self) -> Markup {
        html! {
            div.nav-group {
                a.nav-item.hover.white href = (self.item.link) {
                    (self.item.content)
                    @if !self.sub_levels.is_empty() {
                        i.fas.fa-sort-down style = "height: 50%; padding-left: 5px" {}
                    }
                }
                @if !self.sub_levels.is_empty() {
                    ul.nav-hover-dropdown {
                        @for sub_item in &self.sub_levels {
                            li {
                                a.white.hover href = (sub_item.link) { (sub_item.content)}
                            }
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
                    div.nav-icon style = "margin-right: auto" {
                        a href = "/" {
                            img src = (self.logo_path) style="height:15px";
                        }
                    }
                    @for item in &self.items {
                        (item)
                    }
                    div.nav-item.collapse-button {
                        div.hamburger.hover {
                            input type="checkbox"{}
                            span{}
                            span{}
                            span{}
                        }
                    }
                }
                div {} // artificial spacing
            }
        }
    }
}
