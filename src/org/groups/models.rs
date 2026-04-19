use diesel::{
    Connection, ExpressionMethods, QueryDsl, RunQueryDsl, Selectable, SelectableHelper,
    SqliteExpressionMethods,
    prelude::{AsChangeset, Associations, Identifiable, Insertable, Queryable},
    query_dsl::methods::FindDsl,
    sqlite::Sqlite,
};
use std::error::Error;

use crate::org::{
    DatabaseConnection, context_permissions::context_permissions, contexts::contexts,
    group_users::group_users, users::UserId,
};

use super::schema::groups;

#[derive(DieselNewType, Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub struct GroupId(i32);

impl GroupId {
    // everyone group has id 0
    pub fn everyone() -> Self {
        Self(0)
    }
}

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
    description: String,
    owner_id: UserId,
    admin_group_id: Option<GroupId>,
}

#[derive(Insertable)]
#[diesel(table_name = groups)]
#[diesel(check_for_backend(Sqlite))]
struct NewGroup {
    name: String,
    description: String,
    owner_id: UserId,
    admin_group_id: Option<GroupId>,
}

#[derive(AsChangeset)]
#[diesel(table_name = groups)]
#[diesel(check_for_backend(Sqlite))]
pub struct UpdateGroup {
    name: Option<String>,
    description: Option<String>,
    owner_id: Option<UserId>,
    admin_group_id: Option<Option<GroupId>>,
}

impl UpdateGroup {
    pub fn is_empty(&self) -> bool {
        self.name.is_none()
            && self.description.is_none()
            && self.owner_id.is_none()
            && self.admin_group_id.is_none()
    }
}

impl Group {
    pub fn id(&self) -> GroupId {
        self.group_id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn owner(&self) -> UserId {
        self.owner_id
    }

    pub fn admin_group(&self) -> Option<GroupId> {
        self.admin_group_id
    }
}

impl Group {
    pub fn create_new(
        conn: &mut DatabaseConnection,
        name: String,
        description: String,
        owner_id: UserId,
        admin_group_id: Option<GroupId>,
    ) -> Result<Self, Box<dyn Error>> {
        let new_group = NewGroup {
            name,
            description,
            owner_id,
            admin_group_id,
        };

        Ok(diesel::insert_into(groups::table)
            .values(&new_group)
            .returning(Group::as_returning())
            .get_result(conn)?)
    }

    pub fn delete(self, conn: &mut DatabaseConnection) -> Result<(), Box<dyn Error>> {
        tracing::debug!("{}", self.name());
        conn.transaction(|conn| {
            diesel::delete(group_users::table)
                .filter(group_users::group_id.eq(self.group_id))
                .execute(conn)?;
            diesel::delete(context_permissions::table)
                .filter(context_permissions::group_id.eq(self.group_id))
                .execute(conn)?;
            diesel::update(contexts::table)
                .filter(contexts::admin_group_id.is(self.group_id))
                .set(contexts::admin_group_id.eq(None::<GroupId>))
                .execute(conn)?;
            diesel::update(groups::table)
                .filter(groups::admin_group_id.is(self.group_id))
                .set(groups::admin_group_id.eq(None::<GroupId>))
                .execute(conn)?;
            diesel::delete(FindDsl::find(groups::table, self.group_id)).execute(conn)?;
            Ok::<_, diesel::result::Error>(())
        })?;

        Ok(())
    }

    /// - Ok(Some) if found
    /// - Ok(None) if not found
    pub fn find(
        conn: &mut DatabaseConnection,
        group_id: GroupId,
    ) -> Result<Option<Self>, Box<dyn Error>> {
        match groups::table
            .filter(groups::group_id.eq(group_id))
            .select(Group::as_select())
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
        match groups::table
            .filter(groups::name.eq(name))
            .select(Group::as_select())
            .first(conn)
        {
            Ok(group) => Ok(Some(group)),
            Err(diesel::NotFound) => Ok(None),
            Err(err) => Err(err.into()),
        }
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

    /// write update to group info
    pub fn update(
        conn: &mut DatabaseConnection,
        group_id: GroupId,
        name: Option<String>,
        desc: Option<String>,
        owner_id: Option<UserId>,
        admin_group_id: Option<Option<GroupId>>,
    ) -> Result<(), Box<dyn Error>> {
        let changeset = UpdateGroup {
            name,
            description: desc,
            owner_id,
            admin_group_id,
        };

        if changeset.is_empty() {
            return Ok(());
        }

        diesel::update(groups::table.filter(groups::group_id.eq(group_id)))
            .set(changeset)
            .execute(conn)?;

        Ok(())
    }
}
