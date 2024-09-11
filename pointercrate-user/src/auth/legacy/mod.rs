use pointercrate_core::error::CoreError;

use crate::{error::UserError, User};

use super::AuthenticatedUser;

#[cfg(feature = "legacy_accounts")]
pub use post::Registration;

mod get;
mod patch;
mod post;

pub struct LegacyAuthenticatedUser {
    user: User,
    password_hash: String,
}

impl LegacyAuthenticatedUser {
    pub fn into_user(self) -> User {
        self.user
    }

    pub fn user(&self) -> &User {
        &self.user
    }

    pub(super) fn salt(&self) -> Vec<u8> {
        let raw_parts: Vec<_> = self.password_hash.split('$').filter(|s| !s.is_empty()).collect();

        match &raw_parts[..] {
            [_, _, hash] => b64::decode(&hash[..22]),
            _ => unreachable!(),
        }
    }

    pub(super) fn verify(&self, password: &str) -> Result<(), UserError> {
        let valid = bcrypt::verify(password, &self.password_hash)
            .inspect_err(|bcrypt_err| {
                log::error!(
                    "Internal Error during password verification for account {}: {:?}",
                    self.user,
                    bcrypt_err
                )
            })
            .map_err(|_| UserError::Core(CoreError::Unauthorized))?;

        if valid {
            log::debug!("Password correct, proceeding");

            Ok(())
        } else {
            log::warn!("Wrong password for account {}", self.user);

            Err(UserError::Core(CoreError::Unauthorized))
        }
    }

    pub fn validate_password(password: &str) -> Result<(), UserError> {
        if password.len() < 10 {
            return Err(UserError::InvalidPassword);
        }

        Ok(())
    }
}

impl AuthenticatedUser {
    pub fn legacy(user: User, password_hash: String) -> Self {
        AuthenticatedUser::Legacy(LegacyAuthenticatedUser { user, password_hash })
    }
}

// This code is copied from https://github.com/Keats/rust-bcrypt/blob/master/src/b64.rs
// with slight modifications (removal of `encode` and error handling)
mod b64 {
    use std::collections::HashMap;

    use base64::{engine::general_purpose::STANDARD, Engine};
    use lazy_static::lazy_static;

    // Decoding table from bcrypt base64 to standard base64 and standard -> bcrypt
    // Bcrypt has its own base64 alphabet
    // ./ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789
    lazy_static! {
        #[allow(unused_results)]
        static ref BCRYPT_TO_STANDARD: HashMap<char, &'static str> = {
            let mut m = HashMap::new();
            m.insert('/', "B");
            m.insert('.', "A");
            m.insert('1', "3");
            m.insert('0', "2");
            m.insert('3', "5");
            m.insert('2', "4");
            m.insert('5', "7");
            m.insert('4', "6");
            m.insert('7', "9");
            m.insert('6', "8");
            m.insert('9', "/");
            m.insert('8', "+");
            m.insert('A', "C");
            m.insert('C', "E");
            m.insert('B', "D");
            m.insert('E', "G");
            m.insert('D', "F");
            m.insert('G', "I");
            m.insert('F', "H");
            m.insert('I', "K");
            m.insert('H', "J");
            m.insert('K', "M");
            m.insert('J', "L");
            m.insert('M', "O");
            m.insert('L', "N");
            m.insert('O', "Q");
            m.insert('N', "P");
            m.insert('Q', "S");
            m.insert('P', "R");
            m.insert('S', "U");
            m.insert('R', "T");
            m.insert('U', "W");
            m.insert('T', "V");
            m.insert('W', "Y");
            m.insert('V', "X");
            m.insert('Y', "a");
            m.insert('X', "Z");
            m.insert('Z', "b");
            m.insert('a', "c");
            m.insert('c', "e");
            m.insert('b', "d");
            m.insert('e', "g");
            m.insert('d', "f");
            m.insert('g', "i");
            m.insert('f', "h");
            m.insert('i', "k");
            m.insert('h', "j");
            m.insert('k', "m");
            m.insert('j', "l");
            m.insert('m', "o");
            m.insert('l', "n");
            m.insert('o', "q");
            m.insert('n', "p");
            m.insert('q', "s");
            m.insert('p', "r");
            m.insert('s', "u");
            m.insert('r', "t");
            m.insert('u', "w");
            m.insert('t', "v");
            m.insert('w', "y");
            m.insert('v', "x");
            m.insert('y', "0");
            m.insert('x', "z");
            m.insert('z', "1");
            m
        };
    }

    // Can potentially panic if the hash given contains invalid characters
    pub(super) fn decode(hash: &str) -> Vec<u8> {
        let mut res = String::with_capacity(hash.len());
        for ch in hash.chars() {
            res.push_str(BCRYPT_TO_STANDARD.get(&ch).unwrap())
        }

        // Bcrypt base64 has no padding but standard has
        // so we need to actually add padding ourselves
        if hash.len() % 4 > 0 {
            let padding = 4 - hash.len() % 4;
            for _ in 0..padding {
                res.push('=');
            }
        }

        // if we had non standard chars, it would have errored before
        STANDARD.decode(&res).unwrap()
    }
}
