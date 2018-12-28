use super::{Permissions, User};
use crate::{
    error::PointercrateError,
    operation::{Paginate, Paginator},
    schema::members,
    Result,
};
use diesel::{pg::Pg, query_builder::BoxedSelectStatement, PgConnection, QueryDsl, RunQueryDsl};
use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserPagination {
    #[serde(rename = "before")]
    before_id: Option<i32>,

    #[serde(rename = "after")]
    after_id: Option<i32>,

    limit: Option<i64>,

    name: Option<String>,
    display_name: Option<String>,
    // TODO: this
    has_permissions: Option<Permissions>,
}

impl UserPagination {
    filter_method!(members[
        name = name,
        display_name = display_name
    ]);
}

impl Paginator for UserPagination {
    type QuerySource = members::table;
    type Selection = <crate::model::user::AllColumns as diesel::expression::Expression>::SqlType;

    navigation!(members, member_id, before_id, after_id);

    fn source() -> Self::QuerySource {
        members::table
    }

    fn base<'a>() -> BoxedSelectStatement<'a, Self::Selection, Self::QuerySource, Pg> {
        User::all().into_boxed()
    }
}

impl Paginate<UserPagination> for User {
    fn load(pagination: &UserPagination, connection: &PgConnection) -> Result<Vec<Self>> {
        let mut query = pagination.filter(User::all().into_boxed());

        filter!(query[
            members::member_id > pagination.after_id,
            members::member_id < pagination.before_id
        ]);

        query
            .limit(pagination.limit.unwrap_or(50))
            .load(connection)
            .map_err(PointercrateError::database)
    }
}

/*fn filter<'a, ST>(
    &'a self, mut query: BoxedSelectStatement<'a, ST, members::table, Pg>,
) -> BoxedSelectStatement<'a, ST, members::table, Pg> {
    filter!(query[
        members::name = self.name,
        members::display_name = self.display_name
    ]);

    query
}*/
