use crate::{config::SECRET, error::PointercrateError, schema::members};
use diesel::{expression::bound::Bound, query_dsl::QueryDsl, sql_types, ExpressionMethods};

#[derive(Queryable, Insertable, Debug)]
#[table_name = "members"]
pub struct User {
    #[column_name = "member_id"]
    pub id: i32,

    pub name: String,
    pub display_name: Option<String>,
    pub youtube_channel: Option<String>,

    // TODO: change this to a string PLEASE
    password_hash: Vec<u8>,

    // TODO: remove this
    #[deprecated(note = "I was really fucking stupid when I wrote the database")]
    password_salt: Vec<u8>,

    // TODO: deal with this
    permissions: Vec<u8>,
}

type AllColumns = (
    members::member_id,
    members::name,
    members::display_name,
    members::youtube_channel,
    members::password_hash,
    members::password_salt,
    members::permissions,
);

const ALL_COLUMNS: AllColumns = (
    members::member_id,
    members::name,
    members::display_name,
    members::youtube_channel,
    members::password_hash,
    members::password_salt,
    members::permissions,
);

type All = diesel::dsl::Select<members::table, AllColumns>;

type WithName<'a> = diesel::dsl::Eq<members::name, Bound<sql_types::Text, &'a str>>;
type ByName<'a> = diesel::dsl::Filter<All, WithName<'a>>;

type WithId = diesel::dsl::Eq<members::member_id, Bound<sql_types::Int4, i32>>;
type ById = diesel::dsl::Filter<All, WithId>;

impl User {
    pub fn all() -> All {
        members::table.select(ALL_COLUMNS)
    }

    pub fn by_name(name: &str) -> ByName {
        User::all().filter(members::name.eq(name))
    }

    pub fn by_id(id: i32) -> ById {
        User::all().filter(members::member_id.eq(id))
    }

    // ALRIGHT. the following code is really fucking weird. Here's why:
    // - I need to keep backwards-compatibility with the python code I wrote 2 years ago
    // - Said python code was based on some misunderstanding about bcrypt
    // - The key tokens are signed with is a part of the bcrypt hash of the users password (the salt)
    // concatenated with the app's secret key - I store the bcrypt hashes as BYTEA
    // - I use the non-base64 encoded salt as part of the token key
    // All this leads to the following fucked up code.

    pub fn validate_token(self, token: &str) -> Result<Self, PointercrateError> {
        let secret = {
            let mut vec = SECRET.clone();
            vec.extend(&self.password_salt());
            vec
        };
        let (signing_input, signature) = {
            let split = token.rsplitn(2, ' ').collect::<Vec<_>>();
            if split.len() != 2 {
                return Err(PointercrateError::Unauthorized)
            }
            (split[0], split[1])
        };

        jsonwebtoken::verify(signature, signing_input, &secret, jsonwebtoken::Algorithm::HS256)
            .map_err(|_| PointercrateError::Unauthorized)
            .map(move |_| self)
    }

    fn password_hash(&self) -> String {
        String::from_utf8(self.password_hash.clone()).unwrap()
    }

    pub fn password_salt(&self) -> Vec<u8> {
        let hash = self.password_hash();
        let raw_parts: Vec<_> = hash.split('$').filter(|s| !s.is_empty()).collect();

        match &raw_parts[..] {
            [_, _, hash] => b64::decode(&hash[..22]),
            _ => unreachable!(),
        }
    }

    pub fn verify_password(self, password: &str) -> Result<Self, PointercrateError> {
        let valid = bcrypt::verify(&password, &self.password_hash()).map_err(|_| PointercrateError::Unauthorized)?;

        if valid {
            Ok(self)
        } else {
            Err(PointercrateError::Unauthorized)
        }
    }
}

// This code is copied from https://github.com/Keats/rust-bcrypt/blob/master/src/b64.rs
// with slight modifications (removal of `encode` and error handling)
mod b64 {
    use std::collections::HashMap;

    use base64;

    use lazy_static::lazy_static;

    // Decoding table from bcrypt base64 to standard base64 and standard -> bcrypt
    // Bcrypt has its own base64 alphabet
    // ./ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789
    lazy_static! {
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
                res.push_str("=");
            }
        }

        // if we had non standard chars, it would have errored before
        base64::decode(&res).unwrap()
    }
}
