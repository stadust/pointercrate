use crate::{
    model::{demonlist::record::EmbeddedRecordPD, Model},
    schema::submitters,
};
use derive_more::Display;
use diesel::{insert_into, pg::PgConnection, query_dsl::RunQueryDsl, result::QueryResult};
use ipnetwork::IpNetwork;
use serde_derive::Serialize;

mod get;
mod paginate;
mod patch;

pub use self::{paginate::SubmitterPagination, patch::PatchSubmitter};
use crate::model::By;
use std::hash::{Hash, Hasher};

#[derive(Queryable, Debug, Identifiable, Serialize, Hash, Display, Copy, Clone)]
#[table_name = "submitters"]
#[primary_key("submitter_id")]
#[display(fmt = "{} (Banned: {})", id, banned)]
pub struct Submitter {
    pub id: i32,
    pub ip: IpNetwork,
    pub banned: bool,
}

#[derive(Debug, Serialize, Display)]
#[display(fmt = "{}", submitter)]
pub struct SubmitterWithRecords {
    #[serde(flatten)]
    submitter: Submitter,
    records: Vec<EmbeddedRecordPD>,
}

impl Hash for SubmitterWithRecords {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.submitter.hash(state)
    }
}

#[derive(Insertable, Debug)]
#[table_name = "submitters"]
struct NewSubmitter<'a> {
    #[column_name = "ip_address"]
    ip: &'a IpNetwork,
}

impl By<submitters::ip_address, &IpNetwork> for Submitter {}
impl By<submitters::submitter_id, i32> for Submitter {}

impl Submitter {
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
