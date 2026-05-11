use std::fmt::Display;

use diesel::{
    ExpressionMethods, RunQueryDsl, Selectable,
    prelude::{Associations, Identifiable, Insertable, Queryable},
    sqlite::Sqlite,
};

use crate::org::{
    DatabaseConnection, context_permissions::schema::context_permissions, contexts::ContextId,
    groups::GroupId, permissions::PermissionId,
};

#[derive(DieselNewType, Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub struct ContextPermissionPriority(i32);

impl Display for ContextPermissionPriority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.0))
    }
}

impl ContextPermissionPriority {
    pub fn new(value: i32) -> Self {
        Self(value)
    }

    pub fn value(&self) -> i32 {
        self.0
    }
}

/// value of a permission in a context
#[derive(Selectable, Queryable, Insertable, Identifiable, Associations)]
#[diesel(check_for_backend(Sqlite))]
#[diesel(table_name = context_permissions)]
#[diesel(primary_key(permission_id, context_id))]
#[diesel(belongs_to(crate::org::contexts::Context, foreign_key = context_id))]
#[diesel(belongs_to(crate::org::groups::Group, foreign_key = group_id))]
#[diesel(belongs_to(crate::org::permissions::Permission, foreign_key = permission_id))]
pub struct ContextPermission {
    permission_id: PermissionId,
    group_id: GroupId,
    context_id: ContextId,
    priority: ContextPermissionPriority,
    allowed: bool,
}

impl ContextPermission {
    pub fn set(
        conn: &mut DatabaseConnection,
        permission_id: PermissionId,
        group_id: GroupId,
        context_id: ContextId,
        priority: ContextPermissionPriority,
        allowed: bool,
    ) -> Result<(), diesel::result::Error> {
        diesel::replace_into(context_permissions::table)
            .values(ContextPermission {
                permission_id,
                group_id,
                context_id,
                priority,
                allowed,
            })
            .on_conflict_do_nothing()
            .execute(conn)?;
        Ok(())
    }

    pub fn unset(
        conn: &mut DatabaseConnection,
        permission_id: PermissionId,
        group_id: GroupId,
        context_id: ContextId,
    ) -> Result<(), diesel::result::Error> {
        diesel::delete(context_permissions::table)
            .filter(context_permissions::context_id.eq(context_id))
            .filter(context_permissions::group_id.eq(group_id))
            .filter(context_permissions::permission_id.eq(permission_id))
            .execute(conn)?;
        Ok(())
    }
}
