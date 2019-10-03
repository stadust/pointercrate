use super::{Permissions, User};
use crate::{
    context::RequestContext,
    model::Model,
    operation::{Paginate, Paginator, PaginatorQuery, TablePaginator},
    schema::members,
    Result,
};
use diesel::QueryDsl;
use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserPagination {
    #[serde(rename = "before")]
    before_id: Option<i32>,

    #[serde(rename = "after")]
    after_id: Option<i32>,

    limit: Option<u8>,

    name: Option<String>,
    display_name: Option<String>,
    has_permissions: Option<Permissions>,
}

impl TablePaginator for UserPagination {
    type ColumnType = i32;
    type PaginationColumn = members::member_id;
    type Table = members::table;

    fn query(&self, _: RequestContext) -> PaginatorQuery<members::table> {
        let mut query = User::boxed_all();

        filter!(query[
            members::name = self.name,
            members::display_name = self.display_name
        ]);

        query
    }
}

delegate_to_table_paginator!(UserPagination);

impl Paginate<UserPagination> for User {
    fn load(pagination: &UserPagination, ctx: RequestContext) -> Result<Vec<Self>> {
        ctx.check_permissions(perms!(Administrator))?;

        let mut query = pagination.query(ctx);

        // FIXME: this needs to happen in the filter method!
        if let Some(permissions) = pagination.has_permissions {
            query = query.filter(diesel::dsl::sql(&format!(
                "permissions & {0}::Bit(16) = {0}::Bit(16)",
                permissions.bits()
            )));
        }

        filter!(query[
            members::member_id > pagination.after_id,
            members::member_id < pagination.before_id
        ]);

        pagination_result!(query, pagination, members::member_id, ctx.connection())
    }
}
