pub use fluent::FluentValue;
use fluent::{concurrent::FluentBundle, FluentArgs, FluentResource};
use std::{
    collections::HashMap,
    fs::{read_dir, read_to_string, DirEntry},
    path::{Path, PathBuf},
    sync::OnceLock,
};
use tokio::task_local;
use unic_langid::LanguageIdentifier;

pub struct LocalesLoader {
    locales: HashMap<&'static LanguageIdentifier, FluentBundle<FluentResource>>,
}

impl LocalesLoader {
    pub fn load<P: AsRef<Path>>(langs: &'static [LanguageIdentifier], resource_dirs: Vec<P>) -> Self {
        let mut locales: HashMap<&'static LanguageIdentifier, FluentBundle<FluentResource>> = HashMap::new();

        for path in resource_dirs {
            let locale_dirs: Vec<(&LanguageIdentifier, DirEntry)> = read_dir(path)
                .unwrap()
                .filter_map(|s| s.ok())
                .filter(|entry| {
                    entry.path().is_dir()
                        && langs // ensure we only search through the directories which contain ftl files for a language in `langs`
                            .iter()
                            .any(|lang| entry.file_name().to_str().unwrap() == lang.language.as_str())
                })
                .filter_map(|entry| {
                    Some((
                        langs
                            .iter()
                            .find(|lang| entry.file_name().to_str() == Some(lang.language.as_str()))?,
                        entry,
                    ))
                })
                .collect();

            for (lang, locale_dir) in locale_dirs {
                let bundle = locales.entry(lang).or_insert(FluentBundle::new_concurrent(vec![lang.clone()]));
                let resources: Vec<PathBuf> = read_dir(locale_dir.path())
                    .unwrap()
                    .filter_map(|s| s.ok())
                    .map(|entry| entry.path())
                    .filter(|path| path.is_file())
                    .collect();

                resources.iter().for_each(|path| {
                    let source = read_to_string(path).unwrap();

                    // overriding is enabled so it's possible to have one directory's ftl file's keys override the
                    // keys of files in another loaded directory
                    bundle.add_resource_overriding(FluentResource::try_new(source).unwrap());
                });
            }
        }

        LocalesLoader { locales }
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
        self.locales.get(lang)
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

pub fn get_locale(code: &str) -> Option<&'static LanguageIdentifier> {
    LocalesLoader::get().locales.keys().copied().find(|locale| locale.language == code)
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
