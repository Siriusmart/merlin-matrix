use std::error::Error;

use diesel::{
    ExpressionMethods, JoinOnDsl, NullableExpressionMethods, OptionalExtension,
    QueryDsl, RunQueryDsl, SqliteExpressionMethods,
};

use crate::org::{
    DatabaseConnection,
    context_permissions::context_permissions,
    group_users::{GroupUser, group_users},
    groups::GroupId,
    permissions::permissions,
    rooms::rooms,
    users::{UserId, users},
};

diesel::allow_tables_to_appear_in_same_query!(
    rooms,
    context_permissions,
    users,
    group_users,
    permissions,
);

/// check if user has a specified permission in a context
pub fn user_has_permission(
    conn: &mut DatabaseConnection,
    m_user_id: &str,
    m_user_homeserver: &str,
    m_room_id: &str,
    m_room_homeserver: &str,
    permission_qualifier: &str,
) -> Result<Option<bool>, Box<dyn Error>> {
    let user_groups = users::table
        .inner_join(group_users::table.on(group_users::user_id.eq(users::user_id)))
        .filter(users::m_user_id.eq(m_user_id))
        .filter(users::m_user_homeserver.eq(m_user_homeserver))
        .select(group_users::group_id)
        .distinct();

    Ok(rooms::table
        .inner_join(
            context_permissions::table.on(context_permissions::context_id
                .nullable()
                .is(rooms::context_id)),
        )
        .inner_join(
            permissions::table
                .on(permissions::permission_id.eq(context_permissions::permission_id)),
        )
        .filter(permissions::qualifier.eq(permission_qualifier))
        .filter(rooms::m_room_id.eq(m_room_id))
        .filter(rooms::m_room_homeserver.eq(m_room_homeserver))
        .filter(context_permissions::group_id.eq_any(user_groups))
        .order_by(context_permissions::priority.asc())
        .then_order_by(context_permissions::permission_id.asc())
        .select(context_permissions::allowed)
        .first::<bool>(conn)
        .optional()?)
}

/// add user to group, return Ok(true) is user is previously not in group and now added to group
pub fn add_user_to_group(
    conn: &mut DatabaseConnection,
    user_id: UserId,
    group_id: GroupId,
) -> Result<bool, Box<dyn Error>> {
    let inserted = diesel::insert_into(group_users::table)
        .values(&GroupUser::new(user_id, group_id))
        .on_conflict((group_users::user_id, group_users::group_id))
        .do_nothing()
        .execute(conn)?;

    Ok(inserted != 0)
}
