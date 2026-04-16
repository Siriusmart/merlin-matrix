use diesel::{
    Selectable,
    prelude::{Associations, Identifiable, Insertable, Queryable},
    sqlite::Sqlite,
};

use crate::org::{group_users::schema::group_users, groups::GroupId, users::UserId};

#[derive(Selectable, Queryable, Insertable, Identifiable, Associations)]
#[diesel(check_for_backend(Sqlite))]
#[diesel(table_name = group_users)]
#[diesel(primary_key(user_id, group_id))]
#[diesel(belongs_to(crate::org::users::User, foreign_key = user_id))]
#[diesel(belongs_to(crate::org::groups::Group, foreign_key = group_id))]
pub struct GroupUser {
    user_id: UserId,
    group_id: GroupId,
}

impl GroupUser {
    pub fn new(user_id: UserId, group_id: GroupId) -> Self {
        Self { user_id, group_id }
    }
}
