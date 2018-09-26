use lazy_static::lazy_static;
use std::{fmt::Debug, fs::File, io::Read, str::FromStr};

lazy_static! {
    pub static ref LIST_SIZE: i16 = from_env_or_default("LIST_SIZE", 50);
    pub static ref EXTENDED_LIST_SIZE: i16 = from_env_or_default("EXTENDED_LIST_SIZE", 100);
    pub static ref SECRET: Vec<u8> = {
        let path: String = from_env_or_default("SECRET_FILE", ".secret".into());
        let file = File::open(path).unwrap();
        file.bytes().collect::<Result<Vec<u8>, _>>().unwrap()
    };
}

fn from_env_or_default<T: FromStr>(key: &str, default: T) -> T
where
    <T as FromStr>::Err: Debug,
{
    match std::env::var(key) {
        Ok(value) => value.parse().unwrap(),
        Err(err) => default,
    }
}
