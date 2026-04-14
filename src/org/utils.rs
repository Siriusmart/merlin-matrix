use std::error::Error;

use diesel::{
    ExpressionMethods, JoinOnDsl, NullableExpressionMethods, OptionalExtension, QueryDsl,
    RunQueryDsl, SqliteExpressionMethods,
};

use crate::org::{
    DatabasePool, context_permissions::context_permissions, group_users::group_users,
    permissions::permissions, rooms::rooms, users::users,
};

diesel::allow_tables_to_appear_in_same_query!(
    rooms,
    context_permissions,
    users,
    group_users,
    permissions,
);

pub fn user_has_permission(
    pool: &DatabasePool,
    m_user_id: &str,
    m_user_homeserver: &str,
    m_room_id: &str,
    m_room_homeserver: &str,
    permission_qualifier: &str,
) -> Result<Option<bool>, Box<dyn Error>> {
    let mut conn = pool.get().unwrap();

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
        .first::<bool>(&mut conn)
        .optional()?)
}
