use super::{All, Model};
use crate::{
    bitstring::Bits,
    config::SECRET,
    error::PointercrateError,
    middleware::auth::{CSRFClaims, Claims},
    permissions::{Permissions, PermissionsSet},
    schema::members,
    Result,
};
use diesel::{expression::bound::Bound, query_dsl::QueryDsl, sql_types, ExpressionMethods};
use log::{debug, warn};
use serde::{ser::SerializeMap, Serialize, Serializer};
use std::{
    fmt::{Display, Formatter},
    hash::{Hash, Hasher},
};

mod delete;
mod get;
mod paginate;
mod patch;
mod post;

pub use self::{
    paginate::UserPagination,
    patch::{PatchMe, PatchUser},
    post::Registration,
};

/// Model representing a user in the database
#[derive(Queryable, Debug, Identifiable)]
#[table_name = "members"]
pub struct User {
    /// The [`User`]'s unique ID. This is used to identify users and cannot be changed.
    pub id: i32,

    /// The [`User`]'s unique username. This is used to log-in and cannot be changed.
    pub name: String,

    password_hash: String,

    permissions: Bits,

    /// A user-customizable name for each [`User`].
    ///
    /// If set to anything other than [`None`], the value set here will be displayed everywhere the
    /// username would be displayed otherwise. This value is not guaranteed to be unique and
    /// cannot be used to identify a user. In particular, this value cannot be used for log-in
    pub display_name: Option<String>,

    /// A user-customizable link to a [YouTube](https://youtube.com) channel
    pub youtube_channel: Option<String>,
}

impl PartialEq for User {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Display for User {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self.display_name {
            Some(ref dn) => write!(f, "{} '{}' (ID: {})", self.name, dn, self.id),
            None => write!(f, "{} (ID: {})", self.name, self.id),
        }
    }
}

impl Hash for User {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.name.hash(state);
        self.display_name.hash(state);
        self.youtube_channel.hash(state);
        self.permissions.hash(state);
    }
}

impl Serialize for User {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(5))?;
        map.serialize_entry("id", &self.id)?;
        map.serialize_entry("name", &self.name)?;
        map.serialize_entry("permissions", &self.permissions().bits())?;
        map.serialize_entry("display_name", &self.display_name)?;
        map.serialize_entry("youtube_channel", &self.youtube_channel)?;
        map.end()
    }
}

impl Model for User {
    type From = members::table;
    type Selection = (
        members::member_id,
        members::name,
        members::password_hash,
        members::permissions,
        members::display_name,
        members::youtube_channel,
    );

    fn from() -> Self::From {
        members::table
    }

    fn selection() -> Self::Selection {
        Self::Selection::default()
    }
}

type WithName<'a> = diesel::dsl::Eq<members::name, Bound<sql_types::Text, &'a str>>;
type ByName<'a> = diesel::dsl::Filter<All<User>, WithName<'a>>;

type WithId = diesel::dsl::Eq<members::member_id, Bound<sql_types::Int4, i32>>;
type ById = diesel::dsl::Filter<All<User>, WithId>;

type WithPermissions = diesel::expression::SqlLiteral<sql_types::Bool>;
type ByPermissions = diesel::dsl::Filter<All<User>, WithPermissions>;

impl User {
    pub fn by_name(name: &str) -> ByName {
        User::all().filter(members::name.eq(name))
    }

    pub fn by_id(id: i32) -> ById {
        User::all().filter(members::member_id.eq(id))
    }

    pub fn by_permissions(permissions: Permissions) -> ByPermissions {
        User::all().filter(diesel::dsl::sql(&format!(
            "permissions & {0}::Bit(16) = {0}::Bit(16)",
            permissions.bits()
        )))
    }

    pub fn name(&self) -> &str {
        match self.display_name {
            Some(ref name) => name,
            None => self.name.as_ref(),
        }
    }

    pub fn permissions(&self) -> Permissions {
        Permissions::from_bitstring(&self.permissions)
    }

    pub fn set_permissions(&mut self, permissions: Permissions) {
        self.permissions = permissions.bitstring()
    }

    pub fn has_any(&self, perms: &PermissionsSet) -> bool {
        let own_perms = self.permissions();

        perms.perms.iter().any(|perm| own_perms & *perm == *perm)
    }

    pub fn list_team_member(&self) -> bool {
        self.has_any(&perms!(ListHelper or ListModerator or ListAdministrator))
    }

    pub fn extended_list_access(&self) -> bool {
        self.has_any(&perms!(ExtendedAccess or ListHelper or ListModerator or ListAdministrator))
    }

    pub fn validate_name(name: &mut String) -> Result<()> {
        if name.len() < 3 || name != name.trim() {
            return Err(PointercrateError::InvalidUsername)
        }

        *name = name.trim().to_string();

        Ok(())
    }

    pub fn validate_password(password: &mut String) -> Result<()> {
        if password.len() < 10 {
            return Err(PointercrateError::InvalidPassword)
        }
        Ok(())
    }

    pub fn validate_channel(channel: &mut Option<String>) -> Result<()> {
        *channel = match channel {
            Some(channel) => Some(crate::video::validate_channel(channel)?),
            None => None,
        };

        Ok(())
    }

    // ALRIGHT. the following code is really fucking weird. Here's why:
    // - I need to keep backwards-compatibility with the python code I wrote 2 years ago
    // - Said python code was based on some misunderstanding about bcrypt
    // - The key tokens are signed with is a part of the bcrypt hash of the users password (the
    // salt) concatenated with the app's secret key - I store the bcrypt hashes as BYTEA
    // - I use the non-base64 encoded salt as part of the token key
    // All this leads to the following fucked up code.

    fn jwt_secret(&self) -> Vec<u8> {
        let mut vec = SECRET.clone();
        vec.extend(&self.password_salt());
        vec
    }

    pub fn generate_token(&self) -> String {
        jsonwebtoken::encode(
            &jsonwebtoken::Header::default(),
            &Claims { id: self.id },
            &self.jwt_secret(),
        )
        .unwrap()
    }

    pub fn validate_token(self, token: &str) -> Result<Self> {
        // TODO: maybe one day do something with this
        let mut validation = jsonwebtoken::Validation::default();
        validation.validate_exp = false;

        jsonwebtoken::decode::<Claims>(token, &self.jwt_secret(), &validation)
            .map_err(|err| {
                warn!("Token validation FAILED for account {}: {}", self.id, err);

                PointercrateError::Unauthorized
            })
            .map(move |_| self)
    }

    pub fn generate_csrf_token(&self) -> String {
        use std::time::{Duration, SystemTime, UNIX_EPOCH};

        let start = SystemTime::now();
        let since_epoch = start.duration_since(UNIX_EPOCH).expect("time went backwards (and this is probably gonna bite me in the ass when it comes to daytimesaving crap)");

        let claim = CSRFClaims {
            id: self.id,
            iat: since_epoch.as_secs(),
            exp: (since_epoch + Duration::from_secs(3600)).as_secs(),
        };

        jsonwebtoken::encode(&jsonwebtoken::Header::default(), &claim, &SECRET).unwrap()
    }

    pub fn validate_csrf_token(&self, token: &str) -> Result<()> {
        jsonwebtoken::decode::<CSRFClaims>(token, &SECRET, &jsonwebtoken::Validation::default())
            .map_err(|err| {
                warn!("Token validation FAILED for account {}: {}", self.id, err);

                PointercrateError::Unauthorized
            })
            .map(|_| ())
    }

    pub fn password_salt(&self) -> Vec<u8> {
        let raw_parts: Vec<_> = self
            .password_hash
            .split('$')
            .filter(|s| !s.is_empty())
            .collect();

        match &raw_parts[..] {
            [_, _, hash] => b64::decode(&hash[..22]),
            _ => unreachable!(),
        }
    }

    pub fn set_password(&mut self, password: &str) {
        // it is safe to unwrap here because the only errors that can happen are
        // 'BcryptError::CostNotAllowed' (won't happen because DEFAULT_COST is obviously allowed)
        // or errors that happen during internally parsing the hash the library itself just
        // generated. Obviously, an error there is a bug in the library, so we might as
        // well panic
        self.password_hash = bcrypt::hash(password, bcrypt::DEFAULT_COST).unwrap();
    }

    pub fn verify_password(self, password: &str) -> Result<Self> {
        debug!("Verifying a password!");

        let valid = bcrypt::verify(&password, &self.password_hash).map_err(|err| {
            warn!(
                "Password verification FAILED for account {}: {}",
                self.id, err
            );

            PointercrateError::Unauthorized
        })?;

        if valid {
            debug!("Password correct, proceeding");

            Ok(self)
        } else {
            warn!(
                "Potentially malicious log-in attempt to account {}",
                self.id
            );

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
