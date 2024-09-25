use crate::util::from_env_or_default;
use log::error;
use std::{fs::File, io::Read};

pub fn database_url() -> String {
    std::env::var("DATABASE_URL").expect("DATABASE_URL is not set")
}

pub fn secret() -> Vec<u8> {
    let path: String = from_env_or_default("SECRET_FILE", ".secret".into());

    match File::open(path) {
        Ok(file) => file.bytes().collect::<Result<Vec<u8>, _>>().unwrap(),
        Err(err) if cfg!(debug_assertions) => {
            // needed for integration tests/CI
            error!(
                "Failed to read secret, using an unsecure default since this is a debug build- {:?}",
                err
            );

            vec![0x0; 64]
        },
        Err(err) => panic!("Unable to open secret file: {:?}", err),
    }
}
