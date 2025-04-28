use std::{collections::HashMap, path::PathBuf};

use maud::{html, Markup};

use crate::navigation::TopLevelNavigationBarItem;

#[derive(Clone)]
pub struct Locale {
    pub iso_code: &'static str,
    pub flag_iso_code: &'static str,
}

impl Locale {
    pub fn flag(&self) -> Markup {
        html! {
            span.flag-icon style = (format!(r#"background-image: url("/static/demonlist/images/flags/{}.svg");"#, &self.flag_iso_code)) {}
        }
    }
}

/// Withholds the site's core localization information
pub struct LocalizationConfiguration {
    default: LocaleSet,
    overrides: HashMap<PathBuf, LocaleSet>,
}

#[derive(Clone)]
pub struct LocaleSet {
    // The preference cookie for this [`LocaleSet`] ("preference-{cookie}")
    pub cookie: &'static str,

    locales: Vec<Locale>,

    // Used to gracefully handle attempts at retrieving nonexistant locales
    fallback: Option<Locale>,
}

impl LocaleSet {
    pub fn new(cookie: &'static str) -> Self {
        LocaleSet {
            cookie,
            locales: Vec::new(),
            fallback: None,
        }
    }

    pub fn with_locale(mut self, iso_code: &'static str, flag_iso_code: &'static str) -> Self {
        self.locales.push(Locale { iso_code, flag_iso_code });
        self.locales.sort_by(|a, b| a.iso_code.cmp(b.iso_code)); // ensure set is sorted alphabetically

        self
    }

    pub fn with_fallback(mut self, iso_code: &'static str, flag_iso_code: &'static str) -> Self {
        self.fallback = Some(Locale { iso_code, flag_iso_code });

        self.with_locale(iso_code, flag_iso_code)
    }

    pub fn by_code(&self, code: String) -> Option<Locale> {
        let locale = self.locales.iter().find(|locale| locale.iso_code == &code);

        if locale.is_some() {
            return locale.cloned();
        }

        self.fallback.clone()
    }
}

impl Default for LocaleSet {
    fn default() -> Self {
        LocaleSet {
            cookie: "locale",
            locales: Vec::new(),
            fallback: None,
        }
    }
}

impl LocalizationConfiguration {
    pub fn with_locale(mut self, iso_code: &'static str, flag_iso_code: &'static str) -> Self {
        self.default = self.default.with_locale(iso_code, flag_iso_code);

        self
    }

    pub fn with_fallback(mut self, iso_code: &'static str, flag_iso_code: &'static str) -> Self {
        self.default = self.default.with_fallback(iso_code, flag_iso_code);

        self
    }

    // Override the [`LocaleSet`] for a specific URI. The demon page may
    // support 5 languages, but your guidelines pages might only support 2
    pub fn with_override(mut self, uri: PathBuf, locale_set: LocaleSet) -> Self {
        self.overrides.insert(uri, locale_set);

        self
    }

    // Retrieve a [`LocaleSet`] associated with a specific URI. If one
    // is not found, then the default [`LocaleSet`] is returned.
    pub fn set_by_uri(&self, uri: PathBuf) -> LocaleSet {
        self.overrides
            .iter()
            .find(|(key, _)| key.components().zip(uri.components()).all(|(a, b)| a == b))
            .map(|(_, locale_set)| locale_set.clone())
            .unwrap_or(self.default.clone())
    }
}

impl Default for LocalizationConfiguration {
    fn default() -> Self {
        LocalizationConfiguration {
            default: LocaleSet::default(),
            overrides: HashMap::new(),
        }
    }
}

pub fn locale_selection_dropdown(active_locale: Locale, locale_set: LocaleSet) -> TopLevelNavigationBarItem {
    let mut dropdown = TopLevelNavigationBarItem::new(
        Some("language-selector"),
        None,
        html! {
            span.flex data-cookie = (locale_set.cookie) {
                (active_locale.flag())
                span #active-language style = "margin-left: 8px" { (active_locale.iso_code.to_uppercase()) }
            }
        },
    );

    for locale in locale_set.locales {
        if locale.iso_code == active_locale.iso_code {
            // this locale is currently selected, don't add it to the dropdown
            continue;
        }

        dropdown = dropdown.with_sub_item(
            None,
            html! {
                span data-flag = (locale.flag_iso_code) data-lang = (locale.iso_code) {
                    (locale.flag())
                    span style = "margin-left: 10px" { (locale.iso_code.to_uppercase()) }
                }
            },
        );
    }

    dropdown
}
