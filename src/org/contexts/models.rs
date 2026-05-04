use std::error::Error;

use diesel::{
    ExpressionMethods, QueryDsl, RunQueryDsl, Selectable, SelectableHelper,
    prelude::{Associations, Identifiable, Insertable, Queryable},
    sqlite::Sqlite,
};

use crate::org::{DatabaseConnection, contexts::schema::contexts, groups::GroupId, users::UserId};

#[derive(DieselNewType, Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub struct ContextId(i32);

/// represents a single context, contexts are permission
/// presets to be applied to rooms
#[derive(Selectable, Queryable, Insertable, Identifiable, Associations)]
#[diesel(check_for_backend(Sqlite))]
#[diesel(table_name = contexts)]
#[diesel(primary_key(context_id))]
#[diesel(belongs_to(crate::org::users::User, foreign_key = owner_id))]
#[diesel(belongs_to(crate::org::groups::Group, foreign_key = admin_group_id))]
pub struct Context {
    context_id: ContextId,
    name: String,
    description: String,
    owner_id: UserId,
    admin_group_id: Option<GroupId>,
}

#[derive(Insertable)]
#[diesel(table_name = contexts)]
#[diesel(check_for_backend(Sqlite))]
struct NewContext {
    name: String,
    description: String,
    owner_id: UserId,
    admin_group_id: Option<GroupId>,
}

impl Context {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn id(&self) -> ContextId {
        self.context_id
    }
}

impl Context {
    pub fn create_new(
        conn: &mut DatabaseConnection,
        name: String,
        description: String,
        owner_id: UserId,
        admin_group_id: Option<GroupId>,
    ) -> Result<Self, Box<dyn Error>> {
        let new_context = NewContext {
            name,
            description,
            owner_id,
            admin_group_id,
        };

        Ok(diesel::insert_into(contexts::table)
            .values(new_context)
            .returning(Context::as_returning())
            .get_result(conn)?)
    }

    /// validate group name: ascii alnum with ._-, and at least one . for workspace depth
    pub fn validate_name(name: &str) -> bool {
        name.contains(".")
            && name.split('.').all(|chunk| !chunk.is_empty())
            && name
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'))
    }

    pub fn desc_max_len() -> usize {
        150
    }

    /// - Ok(Some) if found
    /// - Ok(None) if not found
    pub fn find(
        conn: &mut DatabaseConnection,
        context_id: ContextId,
    ) -> Result<Option<Self>, Box<dyn Error>> {
        match contexts::table
            .filter(contexts::context_id.eq(context_id))
            .select(Context::as_select())
            .first(conn)
        {
            Ok(group) => Ok(Some(group)),
            Err(diesel::NotFound) => Ok(None),
            Err(err) => Err(err.into()),
        }
    }

    /// - Ok(Some) if found
    /// - Ok(None) if not found
    pub fn find_by_name(
        conn: &mut DatabaseConnection,
        name: &str,
    ) -> Result<Option<Self>, Box<dyn Error>> {
        match contexts::table
            .filter(contexts::name.eq(name))
            .select(Context::as_select())
            .first(conn)
        {
            Ok(group) => Ok(Some(group)),
            Err(diesel::NotFound) => Ok(None),
            Err(err) => Err(err.into()),
        }
    }
}
