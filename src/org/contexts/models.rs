use diesel::{
    Selectable,
    prelude::{Associations, Identifiable, Insertable, Queryable},
    sqlite::Sqlite,
};

use crate::org::{groups::GroupId, users::UserId};

#[derive(DieselNewType, Debug, Hash, PartialEq, Eq)]
pub struct ContextId(i32);

#[derive(Selectable, Queryable, Insertable, Identifiable, Associations)]
#[diesel(check_for_backend(Sqlite))]
#[diesel(table_name = super::schema::contexts)]
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
