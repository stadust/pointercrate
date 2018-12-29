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
    has_permissions: Option<Permissions>,
}

impl Paginator for UserPagination {
    type Model = User;
    type PaginationColumn = members::member_id;
    type PaginationColumnType = i32;

    filter_method!(members[
        name = name,
        display_name = display_name
    ]);

    fn page(
        &self, last_on_page: Option<Self::PaginationColumnType>,
        first_on_page: Option<Self::PaginationColumnType>,
    ) -> Self {
        UserPagination {
            before_id: last_on_page.map(|i| i + 1),
            after_id: first_on_page.map(|i| i - 1),
            ..self.clone()
        }
    }

    fn limit(&self) -> i64 {
        self.limit.unwrap_or(50)
    }

    fn before(&self) -> Option<i32> {
        self.before_id
    }

    fn after(&self) -> Option<i32> {
        self.after_id
    }
}

impl Paginate<UserPagination> for User {
    fn load(pagination: &UserPagination, connection: &PgConnection) -> Result<Vec<Self>> {
        let mut query = pagination.filter(User::boxed_all());

        if let Some(permissions) = pagination.has_permissions {
            // FIXME: raw inline SQL is a bad idea
            query = query.filter(diesel::dsl::sql(&format!(
                "permissions & {0}::Bit(16) = {0}::Bit(16)",
                permissions.bits
            )));
        }

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
