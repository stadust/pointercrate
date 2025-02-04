use std::{fs::File, io::Read};

use pointercrate_core::util::from_env_or_default;

pub(crate) fn secret() -> Vec<u8> {
    let path: String = from_env_or_default("SECRET_FILE", ".secret".into());

    match File::open(path) {
        Ok(file) => file.bytes().collect::<Result<Vec<u8>, _>>().unwrap(),
        Err(err) if cfg!(debug_assertions) => {
            // needed for integration tests/CI
            log::error!(
                "Failed to read secret, using an unsecure default since this is a debug build - {:?}",
                err
            );

            vec![0x0; 64]
        },
        Err(err) => panic!("Unable to open secret file: {:?}", err),
    }
}

pub fn google_client_id() -> String {
    std::env::var("GOOGLE_CLIENT_ID").expect("GOOGLE_CLIENT_ID is not set")
}

pub fn discord_client_id() -> String {
    std::env::var("DISCORD_CLIENT_ID").expect("DISCORD_CLIENT_ID is not set")
}

/// Address of the HTTP Server (e.g. `http://localhost` or `https://pointercrate.com`)
/// Must be HTTPS unless it is localhost.
/// Only used by OAuth2 at the moment.
/// Do not include a trailing slash.
///
/// Correct:
/// ```env
/// http://localhost
/// https://example.com
/// ```
///
/// Incorrect:
/// ```env
/// # Not HTTPS
/// http://example.com
/// # Missing `https://`
/// example.com
/// # Trailing slash not allowed
/// https://example.com/
/// ```
pub fn host_url() -> String {
    std::env::var("HOST_URL").expect("HOST_URL is not set")
}
