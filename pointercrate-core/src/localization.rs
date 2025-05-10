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

pub fn tr(text_id: &str) -> String {
    LANGUAGE.with(|lang| LOCALES.lookup(lang, text_id))
}

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
