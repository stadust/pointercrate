use std::{fmt::Debug, fs::File, io::Read, str::FromStr};

fn from_env_or_default<T: FromStr>(key: &str, default: T) -> T
where
    <T as FromStr>::Err: Debug,
{
    match std::env::var(key) {
        Ok(value) => value.parse().unwrap(),
        Err(_) => default,
    }
}

pub fn list_size() -> i16 {
    from_env_or_default("LIST_SIZE", 50)
}

pub fn extended_list_size() -> i16 {
    from_env_or_default("EXTENDED_LIST_SIZE", 100)
}

pub fn secret() -> Vec<u8> {
    let path: String = from_env_or_default("SECRET_FILE", ".secret".into());
    let file = File::open(path).expect("Unable to open secret file");
    file.bytes().collect::<Result<Vec<u8>, _>>().unwrap()
}

pub fn port() -> u16 {
    from_env_or_default("PORT", 8088)
}

pub fn database_url() -> String {
    std::env::var("DATABASE_URL").expect("DATABASE_URL is not set")
}
