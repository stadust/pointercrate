pub use fluent::FluentValue;
use fluent::{concurrent::FluentBundle, FluentArgs, FluentResource};
use std::{collections::HashMap, fs, path::Path, sync::OnceLock};
use tokio::task_local;
use unic_langid::LanguageIdentifier;

pub struct LocalesLoader {
    locales: HashMap<&'static LanguageIdentifier, FluentBundle<FluentResource>>,
    fallback: &'static LanguageIdentifier,
}

impl LocalesLoader {
    pub fn new(fallback: &'static LanguageIdentifier) -> Self {
        LocalesLoader {
            locales: HashMap::new(),
            fallback,
        }
    }

    /// Load a [`Vec`] of fluent resources for a particular language
    pub fn locale<P: AsRef<Path>>(mut self, lang: &'static LanguageIdentifier, resources: Vec<P>) -> Self {
        let mut bundle = FluentBundle::new_concurrent(vec![lang.clone()]);

        resources.iter().for_each(|path| {
            let source = fs::read_to_string(path).unwrap();
            bundle.add_resource(FluentResource::try_new(source).unwrap()).unwrap();
        });

        self.locales.insert(&lang, bundle);

        self
    }

    /// Set the `LOCALES` [`OnceLock`] to use this set of loaded locales
    pub fn commit(self) {
        LOCALES.set(self).unwrap_or_else(|_| panic!("Failed to set LOCALES OnceLock"));
    }

    pub fn get() -> &'static LocalesLoader {
        LOCALES
            .get()
            .expect("Locales were not properly initialized. Please ensure that the locales have been loaded correctly!")
    }

    pub fn get_bundle(&self, lang: &'static LanguageIdentifier) -> Option<&FluentBundle<FluentResource>> {
        self.locales.get(lang).or_else(|| self.locales.get(self.fallback))
    }

    pub fn lookup<'a>(
        &self, lang: &'static LanguageIdentifier, text_id: &str, args: Option<&HashMap<&'static str, FluentValue<'a>>>,
    ) -> String {
        let (key, maybe_attr) = match text_id.split_once(".") {
            Some((key, attr)) => (key, Some(attr)),
            None => (text_id, None),
        };

        let bundle = match self.get_bundle(lang) {
            Some(bundle) => bundle,
            None => return text_id.to_string(),
        };

        let message = match bundle.get_message(key) {
            Some(message) => message,
            None => return text_id.to_string(),
        };

        let pattern = match maybe_attr
            .and_then(|attr| message.get_attribute(attr).and_then(|a| Some(a.value())))
            .or_else(|| message.value())
        {
            Some(pattern) => pattern,
            None => return text_id.to_string(),
        };

        let fluent_args = match args {
            Some(args) => {
                let mut fluent_args = FluentArgs::new();
                args.iter().for_each(|(arg, value)| fluent_args.set(arg.to_string(), value.clone()));

                Some(fluent_args)
            },
            None => None,
        };

        // todo: leverage fluent's formatting error handling for better error messages
        bundle.format_pattern(pattern, fluent_args.as_ref(), &mut Vec::new()).to_string()
    }
}

static LOCALES: OnceLock<LocalesLoader> = OnceLock::new();

task_local! {
    pub static LANGUAGE: &'static LanguageIdentifier;
}

pub fn get_locale(code: &str) -> &'static LanguageIdentifier {
    let locales_loader = LocalesLoader::get();
    locales_loader
        .locales
        .keys()
        .find(|locale| locale.language == code)
        .unwrap_or(&locales_loader.fallback)
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
    LANGUAGE
        .try_with(|lang| LocalesLoader::get().lookup(lang, text_id, None))
        .unwrap_or(format!("Invalid context {}", text_id))
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
        use std::collections::HashMap;
        use $crate::localization::{LANGUAGE, FluentValue, LocalesLoader};

        let mut args_map: HashMap<&'static str, FluentValue<'_>> = HashMap::new();

        $(
            args_map.insert($key, FluentValue::from($value.clone()));
        )*

        LANGUAGE.try_with(|lang| LocalesLoader::get().lookup(lang, $text_id, Some(&args_map))).unwrap_or(format!("Invalid context {}", $text_id))
    }};
}
