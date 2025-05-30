pub use fluent::FluentValue;
pub use fluent_templates::{static_loader, LanguageIdentifier, Loader};
use tokio::task_local;

static_loader! {
    pub static LOCALES = {
        locales: "../locales",
        fallback_language: "en",
        core_locales: "../locales/core.ftl",
    };
}

task_local! {
    pub static LANGUAGE: &'static LanguageIdentifier;
}

pub fn get_locale(code: &str) -> &'static LanguageIdentifier {
    for locale in LOCALES.locales() {
        if locale.language == code {
            return locale;
        }
    }

    LOCALES.fallback()
}

/// Utility function for easily retrieving the current [`LanguageIdentifier`] inside the
/// `task_local!` [`LocalKey`] scope of wherever this is called from.
pub fn task_lang() -> &'static LanguageIdentifier {
    LANGUAGE.with(|lang| *lang)
}

/// A utility function for fetching a translated message associated with the
/// given `text_id`. The language of the returned message depends on the value
/// of the `tokio::task_local!` `LANGUAGE` [`LocalKey`] variable. The translations
/// are stored in the `locales` directory.
///
/// This function call must be nested inside of a [`LocalKey`] scope.
pub fn tr(text_id: &str) -> String {
    LANGUAGE.with(|lang| LOCALES.lookup(lang, text_id))
}

/// Like [`tr`], except this function must be used for fetching translations
/// containing variables.
///
/// Example with English translation:
/// ```ignore
/// assert_eq!(
///     trp!("demon-score", ("percent", 99)),
///     "Demonlist score (99%)",
/// );
/// ```
/// Source text: `demon-score = Demonlist score ({$percent}%)`
#[macro_export]
macro_rules! trp {
    ($text_id:expr $(, ($key:expr, $value:expr) )* $(,)?) => {{
        use std::borrow::Cow;
        use std::collections::HashMap;
        use $crate::localization::{LOCALES, LANGUAGE, FluentValue, Loader};

        let mut args_map: HashMap<Cow<'static, str>, FluentValue<'_>> = HashMap::new();

        $(
            args_map.insert(Cow::Borrowed($key), FluentValue::from($value.clone()));
        )*

        LANGUAGE.with(|lang| LOCALES.lookup_with_args(lang, $text_id, &args_map))
    }};
}
