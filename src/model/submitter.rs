use super::{All, Model};
use crate::{model::record::EmbeddedRecordPD, schema::submitters};
use diesel::{
    expression::bound::Bound,
    insert_into,
    pg::PgConnection,
    query_dsl::{QueryDsl, RunQueryDsl},
    result::QueryResult,
    sql_types, ExpressionMethods,
};
use ipnetwork::IpNetwork;
use serde_derive::Serialize;
use std::fmt::{Display, Formatter};

mod get;
mod paginate;
mod patch;

pub use self::{paginate::SubmitterPagination, patch::PatchSubmitter};

#[derive(Queryable, Debug, Identifiable, Serialize, Hash)]
#[table_name = "submitters"]
#[primary_key("submitter_id")]
pub struct Submitter {
    pub id: i32,
    pub ip: IpNetwork,
    pub banned: bool,
}

#[derive(Debug, Serialize, Hash)]
pub struct SubmitterWithRecords {
    #[serde(flatten)]
    submitter: Submitter,
    records: Vec<EmbeddedRecordPD>,
}

impl Display for Submitter {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}", self.id)
    }
}

#[derive(Insertable, Debug)]
#[table_name = "submitters"]
struct NewSubmitter<'a> {
    #[column_name = "ip_address"]
    ip: &'a IpNetwork,
}

type WithIp<'a> = diesel::dsl::Eq<submitters::ip_address, Bound<sql_types::Inet, &'a IpNetwork>>;
type ByIp<'a> = diesel::dsl::Filter<All<Submitter>, WithIp<'a>>;

type WithId = diesel::dsl::Eq<submitters::submitter_id, Bound<sql_types::Integer, i32>>;
type ById = diesel::dsl::Filter<All<Submitter>, WithId>;

impl Submitter {
    pub fn by_ip(ip: &IpNetwork) -> ByIp {
        Submitter::all().filter(submitters::ip_address.eq(ip))
    }

    pub fn by_id(id: i32) -> ById {
        Submitter::all().filter(submitters::submitter_id.eq(id))
    }

    pub fn insert(ip: &IpNetwork, conn: &PgConnection) -> QueryResult<Submitter> {
        let new = NewSubmitter { ip };

        insert_into(submitters::table).values(&new).get_result(conn)
    }
}

impl Model for Submitter {
    type From = submitters::table;
    type Selection = (
        submitters::submitter_id,
        submitters::ip_address,
        submitters::banned,
    );

    fn from() -> Self::From {
        submitters::table
    }

    fn selection() -> Self::Selection {
        Self::Selection::default()
    }
}
