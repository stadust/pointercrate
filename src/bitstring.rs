//! Adds support for postgreSQL BIT columns to diesel
//!
//! Diesel's current support interprets bitstrings as bytea, which simply produces wrong values in
//! the FromSql impl and errors out in ToSql.

use byteorder::{NetworkEndian, ReadBytesExt, WriteBytesExt};
use diesel::{
    deserialize::{self, FromSql, FromSqlRow, Queryable},
    expression::{bound::Bound, AsExpression},
    pg::Pg,
    query_builder::QueryId,
    row::Row,
    serialize::{self, IsNull, Output, ToSql},
};
use std::io::Write;

#[derive(SqlType, Debug)]
#[postgres(oid = "1560", array_oid = "1561")]
pub struct BitString;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Bits {
    pub length: u32,
    pub bits: Vec<u8>,
}

impl QueryId for BitString {
    type QueryId = BitString;

    const HAS_STATIC_QUERY_ID: bool = true;
}

impl AsExpression<BitString> for Bits {
    type Expression = Bound<BitString, Self>;

    fn as_expression(self) -> Self::Expression {
        Bound::new(self)
    }
}

impl<'a> AsExpression<BitString> for &'a Bits {
    type Expression = Bound<BitString, Self>;

    fn as_expression(self) -> Self::Expression {
        Bound::new(self)
    }
}

impl FromSql<BitString, Pg> for Bits {
    fn from_sql(bytes: Option<&[u8]>) -> deserialize::Result<Self> {
        let mut bytes = not_none!(bytes);

        let len = bytes.read_u32::<NetworkEndian>()?;

        Ok(Bits {
            length: len,
            bits: bytes.to_vec(),
        })
    }
}

impl FromSqlRow<BitString, Pg> for Bits {
    fn build_from_row<T: Row<Pg>>(row: &mut T) -> deserialize::Result<Self> {
        FromSql::<BitString, Pg>::from_sql(row.take())
    }
}

impl ToSql<BitString, Pg> for Bits {
    fn to_sql<W: Write>(&self, out: &mut Output<W, Pg>) -> serialize::Result {
        out.write_u32::<NetworkEndian>(self.length)?;
        out.write(&self.bits[..])
            .map(|_| IsNull::No)
            .map_err(Into::into)
    }
}

impl Queryable<BitString, Pg> for Bits {
    type Row = Self;

    fn build(row: Self::Row) -> Self {
        row
    }
}
