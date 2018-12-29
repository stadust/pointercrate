use super::{Permissions, User};
use crate::{
    error::PointercrateError,
    model::Model,
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

impl Paginator for UserPagination {
    type Model = User;
    type PaginationColumn = members::member_id;
    type PaginationColumnType = i32;

    navigation!(members, member_id, before_id, after_id);

    filter_method!(members[
        name = name,
        display_name = display_name
    ]);
}

impl Paginate<UserPagination> for User {
    fn load(pagination: &UserPagination, connection: &PgConnection) -> Result<Vec<Self>> {
        let mut query = pagination.filter(User::boxed_all());

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
