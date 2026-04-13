use diesel::{
    ExpressionMethods, QueryDsl, RunQueryDsl, Selectable, SelectableHelper,
    prelude::{Associations, Identifiable, Insertable, Queryable},
    query_dsl::methods::FindDsl,
    sqlite::Sqlite,
};
use std::error::Error;

use crate::org::{DatabasePool, users::UserId};

use super::schema::groups;

#[derive(DieselNewType, Debug, Hash, PartialEq, Eq)]
pub struct GroupId(i32);

/// - The user creating the group is by default the owner, ownership can be transferred
/// - After creating the group, the owner of the group is not in the group
/// - Users of the admin group can add, remove group members
/// - The owner can replace/remove the admin group
#[derive(Selectable, Queryable, Insertable, Identifiable, Associations)]
#[diesel(check_for_backend(Sqlite))]
#[diesel(table_name = groups)]
#[diesel(primary_key(group_id))]
#[diesel(belongs_to(crate::org::users::User, foreign_key = owner_id))]
#[diesel(belongs_to(crate::org::groups::Group, foreign_key = admin_group_id))]
pub struct Group {
    group_id: GroupId,
    name: String,
    owner_id: UserId,
    admin_group_id: Option<GroupId>,
}

#[derive(Insertable)]
#[diesel(table_name = groups)]
#[diesel(check_for_backend(Sqlite))]
struct NewGroup {
    name: String,
    owner_id: UserId,
}

impl Group {
    pub fn create_new(
        pool: &DatabasePool,
        name: String,
        owner_id: UserId,
    ) -> Result<Self, Box<dyn Error>> {
        let new_group = NewGroup { name, owner_id };

        let mut conn = pool.get().unwrap();

        Ok(diesel::insert_into(groups::table)
            .values(&new_group)
            .returning(Group::as_returning())
            .get_result(&mut conn)?)
    }

    pub fn change_owner(
        &mut self,
        pool: &DatabasePool,
        owner_id: UserId,
    ) -> Result<(), Box<dyn Error>> {
        let mut conn = pool.get().unwrap();

        diesel::update(&*self)
            .set(groups::owner_id.eq(&owner_id))
            .execute(&mut conn)?;

        self.owner_id = owner_id;

        Ok(())
    }

    pub fn change_admin_group(
        &mut self,
        pool: &DatabasePool,
        admin_group_id: Option<GroupId>,
    ) -> Result<(), Box<dyn Error>> {
        let mut conn = pool.get().unwrap();

        diesel::update(&*self)
            .set(groups::admin_group_id.eq(&admin_group_id))
            .execute(&mut conn)?;

        self.admin_group_id = admin_group_id;

        Ok(())
    }

    pub fn delete(self, pool: &DatabasePool) -> Result<(), Box<dyn Error>> {
        let mut conn = pool.get().unwrap();

        diesel::delete(FindDsl::find(groups::table, self.group_id)).execute(&mut conn)?;

        Ok(())
    }

    /// - Ok(Some) if found
    /// - Ok(None) if not found
    pub fn find(pool: &DatabasePool, group_id: GroupId) -> Result<Option<Self>, Box<dyn Error>> {
        let mut conn = pool.get().unwrap();

        match QueryDsl::find(groups::table, group_id)
            .select(Group::as_select())
            .first(&mut conn)
        {
            Ok(group) => Ok(Some(group)),
            Err(diesel::NotFound) => Ok(None),
            Err(err) => Err(err.into()),
        }
    }
}
