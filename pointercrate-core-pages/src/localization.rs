use std::{collections::HashMap, path::PathBuf};

use maud::{html, Markup};
use unic_langid::LanguageIdentifier;

use crate::navigation::TopLevelNavigationBarItem;

#[derive(Clone)]
pub struct Locale {
    pub lang: &'static LanguageIdentifier,
    pub flag_iso_code: &'static str,
}

impl Locale {
    pub fn flag(&self) -> Markup {
        html! {
            span.flag-icon style = (format!(r#"background-image: url("/static/demonlist/images/flags/{}.svg");"#, &self.flag_iso_code)) {}
        }
    }
}

/// Withholds the site's core localization information.
#[derive(Default)]
pub struct LocalizationConfiguration {
    default: LocaleSet,
    overrides: HashMap<PathBuf, LocaleSet>,
}

/// Represents a collection of [`Locale`] objects associated with a specific
/// URI in the [`LocalizationConfiguration`].
#[derive(Clone)]
pub struct LocaleSet {
    /// The preference cookie for this [`LocaleSet`] (`preference-{cookie}`)
    pub cookie: &'static str,

    pub locales: Vec<Locale>,

    /// Used to gracefully handle attempts at retrieving nonexistant locales
    fallback: Option<Locale>,
}

impl LocaleSet {
    /// Initialize a new [`LocaleSet`] with a `cookie`, which represents the
    /// cookie name the client will send their selected language with.
    ///
    /// The cookie will automatically be prefixed with `preference-`, so passing
    /// `guidelines-locale` as the cookie would result in the backend actually
    /// handling `preference-guidelines-locale`, even though this [`LocaleSet`]'s
    /// `cookie` value would remain unchanged.
    pub fn new(cookie: &'static str) -> Self {
        LocaleSet {
            cookie,
            locales: Vec::new(),
            fallback: None,
        }
    }

    /// Append a new [`Locale`] to this [`LocaleSet`].
    pub fn with_locale(mut self, lang: &'static LanguageIdentifier, flag_iso_code: &'static str) -> Self {
        self.locales.push(Locale { lang, flag_iso_code });
        self.locales.sort_by(|a, b| a.lang.language.cmp(&b.lang.language)); // ensure set is sorted alphabetically

        self
    }

    /// Specify the fallback [`Locale`] for this [`LocaleSet`]. This is used to gracefully
    /// handle attempts at retrieving a non-existant language.
    ///
    /// If a fallback [`Locale`] is already set, it will be overridden.
    pub fn with_fallback(mut self, lang: &'static LanguageIdentifier, flag_iso_code: &'static str) -> Self {
        self.fallback = Some(Locale { lang, flag_iso_code });

        self.with_locale(lang, flag_iso_code)
    }

    /// Returns an owned [`Locale`] whose `iso_code` matches the given `code`.
    /// If one is not found, the fallback [`Locale`] will be returned.
    pub fn by_code(&self, code: String) -> Option<Locale> {
        let locale = self.locales.iter().find(|locale| locale.lang.language.as_str() == &code);

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
    /// Append a [`Locale`] to the default [`LocaleSet`].
    pub fn with_locale(mut self, lang: &'static LanguageIdentifier, flag_iso_code: &'static str) -> Self {
        self.default = self.default.with_locale(lang, flag_iso_code);

        self
    }

    /// Specify the fallback [`Locale`] for the default [`LocaleSet`]. If one is already
    /// set, it will be overridden.
    pub fn with_fallback(mut self, lang: &'static LanguageIdentifier, flag_iso_code: &'static str) -> Self {
        self.default = self.default.with_fallback(lang, flag_iso_code);

        self
    }

    /// Override the [`LocaleSet`] for a specific URI. The demon page may
    /// support 5 languages, but your guidelines pages might only support 2.
    ///
    /// # Example
    /// ```ignore
    /// let localization_config = LocalizationConfiguration::default()
    ///     .with_fallback("en", "us")
    ///     .with_locale("fr", "fr")
    ///     .with_locale("es", "es")
    ///     .with_override(
    ///         PathBuf::from("guidelines/"),
    ///         LocaleSet::new("guidelines-locale")
    ///             .with_fallback("de", "de")
    ///             .with_fallback("ru", "ru"),
    ///     );
    /// ```
    pub fn with_override(mut self, uri: PathBuf, locale_set: LocaleSet) -> Self {
        self.overrides.insert(uri, locale_set);

        self
    }

    /// Retrieve the [`LocaleSet`] associated with a specific URI. If one
    /// is not found, then the default [`LocaleSet`] is returned.
    pub fn set_by_uri(&self, uri: PathBuf) -> LocaleSet {
        self.overrides
            .iter()
            .find(|(key, _)| {
                uri.components().count() >= key.components().count() && key.components().zip(uri.components()).all(|(a, b)| a == b)
            })
            .map(|(_, locale_set)| locale_set.clone())
            .unwrap_or(self.default.clone())
    }
}

pub fn locale_selection_dropdown(active_locale: Locale, locale_set: LocaleSet) -> Option<TopLevelNavigationBarItem> {
    if locale_set.locales.len() < 2 {
        return None;
    }

    let mut dropdown = TopLevelNavigationBarItem::new(
        Some("language-selector"),
        None,
        html! {
            span.flex data-cookie = (locale_set.cookie) {
                (active_locale.flag())
                span #active-language style = "margin-left: 8px" { (active_locale.lang.language.as_str().to_uppercase()) }
            }
        },
    );

    for locale in locale_set.locales {
        if locale.lang == active_locale.lang {
            // this locale is currently selected, don't add it to the dropdown
            continue;
        }

        dropdown = dropdown.with_sub_item(
            None,
            html! {
                span data-flag = (locale.flag_iso_code) data-lang = (locale.lang.language.as_str()) {
                    (locale.flag())
                    span style = "margin-left: 10px" { (locale.lang.language.as_str().to_uppercase()) }
                }
            },
        );
    }

    Some(dropdown)
}
