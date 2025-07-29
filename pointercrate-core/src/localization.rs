use crate::error::log_internal_server_error;
pub use fluent::FluentValue;
use fluent::{concurrent::FluentBundle, FluentArgs, FluentError, FluentMessage, FluentResource};
use fluent_syntax::parser::ParserError;
use std::collections::hash_map::Entry;
use std::{collections::HashMap, fs::read_dir, path::Path, sync::OnceLock};
use tokio::task_local;
use unic_langid::subtags::Language;
use unic_langid::{LanguageIdentifier, LanguageIdentifierError};

static LOCALES: OnceLock<LocaleConfiguration> = OnceLock::new();

pub struct LocalesLoader {
    bundles: HashMap<Language, FluentBundle<FluentResource>>,
}

pub struct LocaleConfiguration {
    bundles: HashMap<Language, FluentBundle<FluentResource>>,
    pub fallback: Language,
}

#[derive(thiserror::Error, Debug)]
pub enum LoaderError {
    #[error("I/O Error while reading ftl files: {0}")]
    Io(#[from] std::io::Error),
    #[error("Encountered directory whose name is not a language identifier: {0}")]
    LanguageIdentifier(#[from] LanguageIdentifierError),
    #[error("Error(s) parsing fluent resource file: {0:?}")]
    FluentParsing(Vec<ParserError>),
    #[error("Fluent Resource Conflict(s): {0:?}")]
    FluentConflict(Vec<FluentError>),
    #[error("The same language is registered with non-equal language identifiers (e.g. en-gb vs en-us)")]
    InconsistentLangIds,
}

impl LocalesLoader {
    pub fn load(resource_dirs: &[impl AsRef<Path>]) -> Result<Self, LoaderError> {
        // Cannot use log::warn in this function, because it gets ran before rocket configures logging.
        let mut bundles = HashMap::<Language, FluentBundle<FluentResource>>::new();

        let mut text_ids = Vec::new();

        for path in resource_dirs {
            for dir_entry in read_dir(path)? {
                let dir_entry = dir_entry?;

                if !dir_entry.path().is_dir() {
                    eprintln!("Expected layout for localization directories is [...]/static/{{lang1,lang2,lang3}}/*.ftl. Unexpectedly found non-directory {:?}, ignoring", dir_entry.path());
                    continue;
                }

                let lang_id = LanguageIdentifier::from_bytes(dir_entry.file_name().as_encoded_bytes())?;

                let bundle = match bundles.entry(lang_id.language) {
                    Entry::Occupied(bundle) => {
                        if bundle.get().locales[0] != lang_id {
                            return Err(LoaderError::InconsistentLangIds);
                        }

                        bundle.into_mut()
                    },
                    Entry::Vacant(entry) => entry.insert(FluentBundle::new_concurrent(vec![lang_id])),
                };

                for ftl_file in read_dir(dir_entry.path())? {
                    let ftl_file = ftl_file?;

                    if !ftl_file.path().is_file() {
                        eprintln!("Expected layout for localization directories is [...]/static/{{lang1,lang2,lang3}}/*.ftl. Unexpectedly found non-file {:?}, ignoring", ftl_file.path());
                        continue;
                    }

                    let source = FluentResource::try_new(std::fs::read_to_string(ftl_file.path())?)
                        .map_err(|(_, errors)| LoaderError::FluentParsing(errors))?;

                    for entry in source.entries() {
                        if let fluent_syntax::ast::Entry::Message(msg) = entry {
                            text_ids.push(msg.id.name.to_string());
                        }
                    }

                    bundle.add_resource(source).map_err(LoaderError::FluentConflict)?
                }
            }
        }

        for bundle in bundles.values() {
            for text_id in &text_ids {
                if !bundle.has_message(text_id) {
                    eprintln!("Localization Files for language {} are missing key {}!", bundle.locales[0], text_id);
                }
            }
        }

        Ok(LocalesLoader { bundles })
    }

    /// Set the `LOCALES` [`OnceLock`] to use this set of loaded locales
    pub fn commit(self, fallback: Language) {
        assert!(self.bundles.contains_key(&fallback));

        let config = LocaleConfiguration {
            bundles: self.bundles,
            fallback,
        };
        LOCALES
            .set(config)
            .unwrap_or_else(|_| panic!("LOCALES OnceLock already initialized"));
    }

    /// Function setting up an empty [`LocaleConfiguration`] that will fail to localize
    /// all keys. Mostly useful for integration tests.
    pub fn empty() {
        // Code assumes that the fallback has an entry in the hashmap, so create a dummy bundle
        let mut bundles = HashMap::new();
        let lang_id = LanguageIdentifier::default();
        let lang = lang_id.language;
        bundles.insert(lang, FluentBundle::new_concurrent(vec![lang_id]));
        let empty = LocaleConfiguration { bundles, fallback: lang };
        _ = LOCALES.set(empty)
    }
}

impl LocaleConfiguration {
    pub fn get() -> &'static Self {
        LOCALES
            .get()
            .expect("Locales were not properly initialized. Please ensure that the locales have been loaded correctly!")
    }

    pub fn active_locale(&self) -> &LanguageIdentifier {
        self.bundles
            .get(&task_lang())
            .map(|bundle| &bundle.locales[0])
            .unwrap_or(&self.bundles[&self.fallback].locales[0])
    }

    /// Returns a [`LanguageIdentifier`] whose string representation matches the given `code`.
    /// If one is not found, the [`LanguageIdentifier`] associated with the fallback language will be returned.
    pub fn by_code(&self, code: &str) -> &LanguageIdentifier {
        // Can unwrap, there's an assertion in `commit()` to assert the fallback language exists
        let fallback_locale = &self.bundles[&self.fallback].locales[0];

        self.locales()
            .find(|lang_id| lang_id.to_string().eq_ignore_ascii_case(code))
            .unwrap_or(fallback_locale)
    }

    pub fn locales(&self) -> impl ExactSizeIterator<Item = &LanguageIdentifier> {
        self.bundles.values().map(|bundle| &bundle.locales[0])
    }

    fn get_message(&self, lang: &Language, text_id: &str) -> Option<(&FluentBundle<FluentResource>, FluentMessage<'_>)> {
        // Can unwrap, there's an assertion in `commit()` to assert the fallback language exists
        let fallback_bundle = &self.bundles[&self.fallback];
        let bundle = self.bundles.get(lang).unwrap_or_else(|| {
            log_internal_server_error(format!("Request for language that has no bundle associated with it: {}", lang));

            fallback_bundle
        });

        bundle
            .get_message(text_id)
            .map(|msg| (bundle, msg))
            .or_else(|| fallback_bundle.get_message(text_id).map(|msg| (bundle, msg)))
    }

    pub fn lookup<'a>(&self, lang: &Language, text_id: &str, args: Option<&HashMap<&str, FluentValue<'a>>>) -> String {
        let (key, maybe_attr) = match text_id.split_once(".") {
            Some((key, attr)) => (key, Some(attr)),
            None => (text_id, None),
        };

        let Some((bundle, message)) = self.get_message(lang, key) else {
            #[cfg(not(test))]
            log_internal_server_error(format!("Invalid fluent key: {}", text_id));

            return text_id.to_string();
        };

        let pattern = match maybe_attr
            .and_then(|attr| message.get_attribute(attr).map(|a| a.value()))
            .or_else(|| message.value())
        {
            Some(pattern) => pattern,
            None => {
                #[cfg(not(test))]
                log_internal_server_error(format!("Invalid fluent attributes for key {}: {:?}", text_id, maybe_attr));

                return text_id.to_string();
            },
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

task_local! {
    pub static LANGUAGE: Language;
}

/// Utility function for easily retrieving the current [`LanguageIdentifier`] inside the
/// `task_local!` [`LocalKey`] scope of wherever this is called from.
pub fn task_lang() -> Language {
    LANGUAGE.with(|lang| *lang)
}

/// A utility function for fetching a translated message associated with the
/// given `text_id`. The language of the returned message depends on the value
/// of the `tokio::task_local!` `LANGUAGE` [`LocalKey`] variable. The translations
/// are stored in the `locales` directory.
///
/// This function call must be nested inside a [`LocalKey`] scope.
pub fn tr(text_id: &str) -> String {
    LANGUAGE
        .try_with(|lang| LocaleConfiguration::get().lookup(lang, text_id, None))
        .unwrap_or_else(|err| {
            log_internal_server_error(format!("Localization Failure: Call tr from invalid context: {:?}", err));

            text_id.to_owned()
        })
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
    ($text_id:literal, $($key:literal = $value:expr ),*) => {{
        use std::collections::HashMap;
        use $crate::localization::{LANGUAGE, FluentValue, LocaleConfiguration};

        let mut args_map: HashMap<&'static str, FluentValue<'_>> = HashMap::new();

        $(
            args_map.insert($key, FluentValue::from($value.clone()));
        )*

        LANGUAGE.try_with(|lang| LocaleConfiguration::get().lookup(lang, $text_id, Some(&args_map)))
        .unwrap_or_else(|err|{
            $crate::error::log_internal_server_error(format!("Localization Failure: Call trp! from invalid context: {:?}", err));

            $text_id.to_owned()
        })
    }};
}
