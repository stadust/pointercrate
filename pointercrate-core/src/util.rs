use serde::{de::Error, Deserialize, Deserializer};
use std::{fmt::Debug, str::FromStr};

pub fn from_env_or_default<T: FromStr>(key: &str, default: T) -> T
where
    <T as FromStr>::Err: Debug,
{
    match std::env::var(key) {
        Ok(value) => value.parse().unwrap(),
        Err(_) => default,
    }
}

#[allow(clippy::option_option)]
pub fn nullable<'de, T, D>(deserializer: D) -> std::result::Result<Option<Option<T>>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Ok(Some(Option::deserialize(deserializer)?))
}

pub fn non_nullable<'de, T, D>(deseralizer: D) -> std::result::Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    match Option::deserialize(deseralizer)? {
        None => Err(<D as Deserializer<'de>>::Error::custom("null value on non-nullable field")),
        some => Ok(some),
    }
}
