use diesel::{
    QueryDsl, RunQueryDsl, Selectable, SelectableHelper,
    prelude::{Insertable, Queryable},
    query_dsl::methods::FindDsl,
    sqlite::Sqlite,
};
use std::error::Error;

use crate::org::DatabasePool;

use super::schema::groups;

#[derive(DieselNewType, Debug)]
pub struct GroupId(i32);

/// - The user creating the group is by default the owner, ownership can be transferred
/// - After creating the group, the owner of the group is not in the group
/// - Users of the admin group can add, remove group members
/// - The owner can replace/remove the admin group
#[derive(Selectable, Queryable, Insertable)]
#[diesel(table_name = super::schema::groups)]
#[diesel(check_for_backend(Sqlite))]
pub struct Group {
    id: i32,
    display: String,
    owner_name: String,
    owner_homeserver: String,
    admin_group_id: Option<GroupId>,
}

#[derive(Insertable)]
#[diesel(table_name = super::schema::groups)]
#[diesel(check_for_backend(Sqlite))]
pub struct NewGroup {
    display: String,
    owner_name: String,
    owner_homeserver: String,
}

impl Group {
    pub fn create_new(
        pool: &DatabasePool,
        display: String,
        owner_name: String,
        owner_homeserver: String,
    ) -> Result<Self, Box<dyn Error>> {
        let new_group = NewGroup {
            display,
            owner_name,
            owner_homeserver,
        };

        let mut conn = pool.get().unwrap();

        Ok(diesel::insert_into(groups::table)
            .values(&new_group)
            .returning(Group::as_returning())
            .get_result(&mut conn)?)
    }

    pub fn change_owner(
        mut self,
        pool: &DatabasePool,
        owner_name: String,
        owner_homeserver: String,
    ) -> Result<Group, Box<dyn Error>> {
        self.owner_name = owner_name;
        self.owner_homeserver = owner_homeserver;

        let mut conn = pool.get().unwrap();

        Ok(diesel::replace_into(groups::table)
            .values(&self)
            .returning(Group::as_returning())
            .get_result(&mut conn)?)
    }

    pub fn change_admin_group(
        mut self,
        pool: &DatabasePool,
        admin_group_id: Option<GroupId>,
    ) -> Result<Group, Box<dyn Error>> {
        self.admin_group_id = admin_group_id;

        let mut conn = pool.get().unwrap();

        Ok(diesel::replace_into(groups::table)
            .values(&self)
            .returning(Group::as_returning())
            .get_result(&mut conn)?)
    }

    pub fn delete(self, pool: &DatabasePool) -> Result<(), Box<dyn Error>> {
        let mut conn = pool.get().unwrap();

        diesel::delete(FindDsl::find(groups::table, self.id)).execute(&mut conn)?;

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
