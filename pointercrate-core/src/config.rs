use crate::util::from_env_or_default;
use std::{fs::File, io::Read};

pub fn database_url() -> String {
    std::env::var("DATABASE_URL").expect("DATABASE_URL is not set")
}

pub fn secret() -> Vec<u8> {
    let path: String = from_env_or_default("SECRET_FILE", ".secret".into());
    let file = File::open(path).expect("Unable to open secret file");
    file.bytes().collect::<Result<Vec<u8>, _>>().unwrap()
}
