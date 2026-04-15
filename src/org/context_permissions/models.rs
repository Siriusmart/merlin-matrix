use diesel::{
    Selectable,
    prelude::{Associations, Identifiable, Insertable, Queryable},
    sqlite::Sqlite,
};

use crate::org::{
    context_permissions::schema::context_permissions, contexts::ContextId, groups::GroupId,
    permissions::PermissionId,
};

#[derive(DieselNewType, Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub struct ContextPermissionPriority(i32);

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
