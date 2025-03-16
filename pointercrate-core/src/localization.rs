use std::{borrow::Cow, collections::HashMap};

use fluent::FluentValue;
use fluent_templates::{static_loader, LanguageIdentifier, Loader};

static_loader! {
    static LOCALES = {
        locales: "../locales",
        fallback_language: "en",
    };
}

pub fn get_locale(code: &str) -> &'static LanguageIdentifier {
    for locale in LOCALES.locales() {
        if locale.language == code {
            return locale;
        }
    }

    LOCALES.fallback()
}

pub fn tr(lang: &'static LanguageIdentifier, text_id: &str) -> String {
    LOCALES.lookup(lang, text_id)
}

pub fn ftr(lang: &'static LanguageIdentifier, text_id: &str, args: Vec<(&'static str, FluentValue)>) -> String {
    let mut args_map: HashMap<Cow<'static, str>, FluentValue<'_>> = HashMap::new();

    for arg in args {
        args_map.insert(Cow::Borrowed(arg.0), arg.1);
    }

    LOCALES.lookup_with_args(lang, text_id, &args_map)
}
