use super::{All, Model};
use crate::{
    bitstring::Bits, config::SECRET, error::PointercrateError, middleware::auth::Claims,
    schema::members, Result,
};
use bitflags::bitflags;
use diesel::{expression::bound::Bound, query_dsl::QueryDsl, sql_types, ExpressionMethods};
use log::debug;
use serde::{
    ser::{SerializeMap, SerializeSeq},
    Serialize, Serializer,
};
use serde_derive::Deserialize;
use std::{
    collections::HashSet,
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

bitflags! {
    /// Permissions bitmask used for authorisation.
    ///
    /// A `Permissions` object can be see as a 16-ary boolean function that evaluate to true if,
    /// and only if, **all** bits that are set in the [`Permissions`] object are also set in the input.
    ///
    /// Consult the [pointercrate API documentation](https://pointercrate.com/documentation#permissions) for more details
    #[derive(Deserialize)]
    pub struct Permissions: u16 {
        const ExtendedAccess = 0b0000_0000_0000_0001;
        const ListHelper = 0b0000_0000_0000_0010;
        const ListModerator = 0b0000_0000_0000_0100;
        const ListAdministrator = 0b0000_0000_0000_1000;
        const LeaderboardModerator = 0b0000_0000_0001_0000;
        const LeaderboardAdministrator = 0b0000_0000_0010_0000;
        const Moderator = 0b0010_0000_0000_0000;
        const Administrator = 0b0100_0000_0000_0000;
        const ItIsImpossibleToGainThisPermission = 0b1000_0000_0000_0000;
    }
}

impl Permissions {
    /// Gets a [`Permissions`] object that has all permissions set that would be required to assign
    /// all the permissions stored in this object
    pub fn assignable_from(self) -> Permissions {
        let mut from = Permissions::empty();

        if (Permissions::ListHelper | Permissions::ListModerator) & self != Permissions::empty() {
            from.insert(Permissions::ListAdministrator)
        }

        if (Permissions::Moderator
            | Permissions::ListModerator
            | Permissions::LeaderboardAdministrator
            | Permissions::ExtendedAccess)
            & self
            != Permissions::empty()
        {
            from.insert(Permissions::Administrator)
        }

        if Permissions::LeaderboardModerator & self != Permissions::empty() {
            from.insert(Permissions::LeaderboardAdministrator)
        }

        if Permissions::Administrator & self != Permissions::empty() {
            from.insert(Permissions::ItIsImpossibleToGainThisPermission)
        }

        from
    }

    /// Gets a [`Permissions`] object containing all the permissions you can assign if you have the
    /// permissions stored in this object.
    pub fn assigns(self) -> Permissions {
        let mut perms = Permissions::empty();

        if Permissions::ListAdministrator & self != Permissions::empty() {
            perms.insert(Permissions::ListHelper | Permissions::ListModerator)
        }

        if Permissions::Administrator & self != Permissions::empty() {
            perms.insert(
                Permissions::Moderator
                    | Permissions::ListAdministrator
                    | Permissions::LeaderboardAdministrator
                    | Permissions::ExtendedAccess,
            )
        }

        if Permissions::LeaderboardAdministrator & self != Permissions::empty() {
            perms.insert(Permissions::LeaderboardModerator)
        }

        perms
    }

    /// Checks whether a user with the current permission set can assign `permissions` to another
    /// user
    pub fn can_assign(self, permissions: Permissions) -> bool {
        self.assigns() & permissions == permissions
    }

    /// Converts a 16-Bit [`Bits`] value to a [`Permissions`] object
    ///
    /// ## Panics
    /// Panics if [`Bits::length`] is unequal to 16
    fn from_bitstring(bits: &Bits) -> Self {
        assert!(bits.length == 16);

        Permissions::from_bits_truncate((bits.bits[0] as u16) << 8 | bits.bits[1] as u16)
    }

    /// Converts this [`Permissions`] object into a [`Bits`] object of length 16
    fn bitstring(self) -> Bits {
        Bits {
            length: 16,
            bits: vec![(self.bits >> 8) as u8, self.bits as u8],
        }
    }
}

impl Serialize for Permissions {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u16(self.bits)
    }
}

/// Struct representing a set of [`Permissions`].
///
/// A [`PermissionsSet`] object can be seen as a boolean function that evaluates to true if, and
/// only if, any of the contained [`Permissions`] objects evaluate to true for the given input.
#[derive(Debug)]
pub struct PermissionsSet {
    /// The contained permissions
    pub perms: HashSet<Permissions>,
}

impl Serialize for PermissionsSet {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.perms.len()))?;
        for e in &self.perms {
            seq.serialize_element(e)?;
        }
        seq.end()
    }
}

impl PermissionsSet {
    /// Construts a singleton [`PermissionsSet`] containing only the given [`Permissions`] object
    pub fn one(perm: Permissions) -> Self {
        let mut set = HashSet::new();

        set.insert(perm);

        PermissionsSet { perms: set }
    }
}

impl Display for PermissionsSet {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        let mut sep = "";

        write!(f, "'")?;

        for perm in &self.perms {
            write!(f, "{}{:?}", sep, perm)?;
            sep = "' or '"
        }

        write!(f, "'")
    }
}

macro_rules! demand_perms {
    ($user: ident, $($($perm: ident),+)or*) => {
        {
            use crate::model::user::{PermissionsSet, Permissions};
            use crate::error::PointercrateError;
            use std::collections::HashSet;

            let mut perm_set = HashSet::new();

            $(
                perm_set.insert($(Permissions::$perm|)+ Permissions::empty());
            )*

            let perm_set = PermissionsSet {
                perms: perm_set
            };

            if !$user.has_any(&perm_set) {
                return Err(PointercrateError::MissingPermissions {
                    required: perm_set
                })
            }

            $user
        }
    }
}

/// Model representing a user in the database
#[derive(Queryable, Debug, Identifiable)]
#[table_name = "members"]
pub struct User {
    /// The [`User`]'s unique ID. This is used to identify users and cannot be changed.
    pub id: i32,

    /// The [`User`]'s unique username. This is used to log-in and cannot be changed.
    pub name: String,

    /// A user-customizable name for each [`User`].
    ///
    /// If set to anything other than [`None`], the value set here will be displayed everywhere the
    /// username would be displayed otherwise. This value is not guaranteed to be unique and
    /// cannot be used to identify a user. In particular, this value cannot be used for log-in
    pub display_name: Option<String>,

    /// A user-customizable link to a [YouTube](https://youtube.com) channel
    pub youtube_channel: Option<String>,

    // TODO: change this to a string PLEASE
    password_hash: Vec<u8>,

    // TODO: remove this
    #[deprecated(note = "I was really fucking stupid when I wrote the database")]
    password_salt: Vec<u8>,

    permissions: Bits,
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
        map.serialize_entry("permissions", &self.permissions().bits)?;
        map.serialize_entry("display_name", &self.display_name)?;
        map.serialize_entry("youtube_channel", &self.youtube_channel)?;
        map.end()
    }
}

impl Model for User {
    type From = members::table;
    type Selection = crate::model::user::AllColumns;

    fn from() -> Self::From {
        members::table
    }

    fn selection() -> Self::Selection {
        ALL_COLUMNS
    }
}

pub type AllColumns = (
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

type WithName<'a> = diesel::dsl::Eq<members::name, Bound<sql_types::Text, &'a str>>;
type ByName<'a> = diesel::dsl::Filter<All<User>, WithName<'a>>;

type WithId = diesel::dsl::Eq<members::member_id, Bound<sql_types::Int4, i32>>;
type ById = diesel::dsl::Filter<All<User>, WithId>;

impl User {
    pub fn by_name(name: &str) -> ByName {
        User::all().filter(members::name.eq(name))
    }

    pub fn by_id(id: i32) -> ById {
        User::all().filter(members::member_id.eq(id))
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
        debug!("Validating a token!");

        let (signing_input, signature) = {
            let split = token.rsplitn(2, '.').collect::<Vec<_>>();
            if split.len() != 2 {
                return Err(PointercrateError::Unauthorized)
            }
            (split[0], split[1])
        };

        jsonwebtoken::verify(
            signature,
            signing_input,
            &self.jwt_secret(),
            jsonwebtoken::Algorithm::HS256,
        )
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

    pub fn set_password(&mut self, password: &str) {
        // it is safe to unwrap here because the only errors that can happen are
        // 'BcryptError::CostNotAllowed' (won't happen because DEFAULT_COST is obviously allowed)
        // or errors that happen during internally parsing the hash the library itself just
        // generated. Obviously, an error there is a bug in the library, so we might as
        // well panic
        self.password_hash = bcrypt::hash(password, bcrypt::DEFAULT_COST)
            .unwrap()
            .into_bytes();
    }

    pub fn verify_password(self, password: &str) -> Result<Self> {
        debug!("Verifying a password!");

        let valid = bcrypt::verify(&password, &self.password_hash())
            .map_err(|_| PointercrateError::Unauthorized)?;

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
