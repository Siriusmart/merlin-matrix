use std::error::Error;

use diesel::{
    RunQueryDsl, Selectable, SelectableHelper,
    prelude::{Associations, Identifiable, Insertable, Queryable},
    sqlite::Sqlite,
};

use crate::org::{DatabasePool, contexts::schema::contexts, groups::GroupId, users::UserId};

#[derive(DieselNewType, Debug, Hash, PartialEq, Eq)]
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
pub struct NewContext {
    name: String,
    description: String,
    owner_id: UserId,
}

impl Context {
    pub fn create_new(
        pool: &DatabasePool,
        name: String,
        description: String,
        owner_id: UserId,
    ) -> Result<Self, Box<dyn Error>> {
        let new_context = NewContext {
            name,
            description,
            owner_id,
        };

        let mut conn = pool.get().unwrap();

        Ok(diesel::insert_into(contexts::table)
            .values(new_context)
            .returning(Context::as_returning())
            .get_result(&mut conn)?)
    }
}
