use crate::org::{
    context_permissions::context_permissions, group_users::group_users, permissions::permissions,
    rooms::rooms, users::users,
};

pub mod contexts;
pub mod groups;

use crate::org::contexts::contexts as contexts_table;
use crate::org::groups::groups as groups_table;

diesel::allow_tables_to_appear_in_same_query!(
    contexts_table,
    groups_table,
    rooms,
    context_permissions,
    users,
    group_users,
    permissions,
);
