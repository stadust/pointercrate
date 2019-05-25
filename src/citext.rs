//! Adds support for postgreSQL CITEXT columns to diesel
//!
//! Diesel currently does not support CITEXT columns at all, and interpreting them as TEXT columns
//! causes problems if you have UNIQUE constraints on CITEXT columns

use derive_more::Display;
use diesel::{
    deserialize::{self, FromSql},
    pg::Pg,
    query_builder::QueryId,
    serialize::{self, IsNull, Output, ToSql},
};
use serde::{Serialize, Serializer};
use serde_derive::Deserialize;
use std::{borrow::Borrow, cmp::Ordering, io::Write, ops::Deref};

#[derive(SqlType, Debug, Copy, Clone)]
#[postgres(type_name = "CITEXT")]
pub struct CiText;

pub type Citext = CiText;

#[derive(Clone, Debug, Hash, AsExpression, FromSqlRow, Serialize, Deserialize, Display)]
#[sql_type = "CiText"]
#[serde(transparent)]
pub struct CiString(pub String);

#[derive(Debug, Hash, AsExpression, Display)]
#[diesel(not_sized)]
#[sql_type = "CiText"]
#[repr(transparent)]
pub struct CiStr(str);

impl Serialize for CiStr {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_ref())
    }
}

impl QueryId for CiText {
    type QueryId = CiText;

    const HAS_STATIC_QUERY_ID: bool = true;
}

impl ToSql<CiText, Pg> for CiString {
    fn to_sql<W: Write>(&self, out: &mut Output<W, Pg>) -> serialize::Result {
        out.write(self.as_bytes())
            .map(|_| IsNull::No)
            .map_err(Into::into)
    }
}

impl<'a> ToSql<CiText, Pg> for &'a CiStr {
    fn to_sql<W: Write>(&self, out: &mut Output<W, Pg>) -> serialize::Result {
        out.write(self.as_bytes())
            .map(|_| IsNull::No)
            .map_err(Into::into)
    }
}

impl FromSql<CiText, Pg> for CiString {
    fn from_sql(bytes: Option<&[u8]>) -> deserialize::Result<Self> {
        Ok(CiString(std::str::from_utf8(not_none!(bytes))?.to_string()))
    }
}

impl PartialEq<CiString> for CiString {
    fn eq(&self, other: &CiString) -> bool {
        self.0.to_lowercase().eq(&other.0.to_lowercase())
    }
}

impl Eq for CiString {}

impl PartialOrd<CiString> for CiString {
    fn partial_cmp(&self, other: &CiString) -> Option<Ordering> {
        self.0.to_lowercase().partial_cmp(&other.0.to_lowercase())
    }
}

impl Ord for CiString {
    fn cmp(&self, other: &CiString) -> Ordering {
        self.0.to_lowercase().cmp(&other.0.to_lowercase())
    }
}

impl AsRef<str> for CiString {
    fn as_ref(&self) -> &str {
        &*self.0
    }
}

impl Borrow<str> for CiString {
    fn borrow(&self) -> &str {
        &*self.0
    }
}

impl Deref for CiString {
    type Target = String;

    fn deref(&self) -> &String {
        &self.0
    }
}

impl AsRef<CiStr> for CiStr {
    fn as_ref(&self) -> &CiStr {
        self
    }
}

impl CiStr {
    pub fn from_str(s: &str) -> &CiStr {
        unsafe { &*(s as *const str as *const CiStr) }
    }
}

impl AsRef<CiStr> for CiString {
    fn as_ref(&self) -> &CiStr {
        CiStr::from_str(self.0.as_ref())
    }
}

impl Deref for CiStr {
    type Target = str;

    fn deref(&self) -> &str {
        &self.0
    }
}

impl PartialEq for CiStr {
    fn eq(&self, other: &CiStr) -> bool {
        self.to_lowercase().eq(&other.to_lowercase())
    }
}

impl Eq for CiStr {}

impl PartialOrd for CiStr {
    fn partial_cmp(&self, other: &CiStr) -> Option<Ordering> {
        self.to_lowercase().partial_cmp(&other.to_lowercase())
    }
}

impl Ord for CiStr {
    fn cmp(&self, other: &CiStr) -> Ordering {
        self.to_lowercase().cmp(&other.to_lowercase())
    }
}

impl Into<String> for CiString {
    fn into(self) -> String {
        self.0
    }
}

impl From<String> for CiString {
    fn from(value: String) -> Self {
        CiString(value)
    }
}
