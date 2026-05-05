use diesel::{
    ExpressionMethods, JoinOnDsl, NullableExpressionMethods, OptionalExtension, QueryDsl,
    RunQueryDsl, SqliteExpressionMethods,
};
use tracing::*;

use crate::org::{
    DatabaseConnection,
    context_permissions::context_permissions,
    contexts::ContextId,
    group_users::group_users::{self, table},
    permissions::permissions,
    rooms::{RoomId, rooms},
    users::users,
};

#[instrument(skip_all)]
pub fn set_room_context(
    conn: &mut DatabaseConnection,
    room_id: RoomId,
    context: Option<ContextId>,
) -> Result<(), diesel::result::Error> {
    trace!("room_id={room_id:?} context={context:?}");
    diesel::update(rooms::table)
        .filter(rooms::room_id.eq(room_id))
        .set(rooms::context_id.eq(context))
        .execute(conn)?;
    Ok(())
}

/// check if user has a specified permission in a context
#[instrument(skip_all)]
pub fn user_has_permission(
    conn: &mut DatabaseConnection,
    m_user_id: &str,
    m_user_homeserver: &str,
    m_room_id: &str,
    permission_qualifier: &str,
) -> Result<Option<bool>, diesel::result::Error> {
    trace!(
        "m_user_id={m_user_id} m_user_homeserver={m_user_homeserver} m_room_id={m_room_id} qualifier={permission_qualifier}"
    );

    let user_groups = users::table
        .inner_join(table.on(group_users::user_id.eq(users::user_id)))
        .filter(users::m_user_id.eq(m_user_id))
        .filter(users::m_user_homeserver.eq(m_user_homeserver))
        .select(group_users::group_id)
        .distinct();

    rooms::table
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
        .filter(context_permissions::group_id.eq_any(user_groups))
        .order_by(context_permissions::priority.asc())
        .then_order_by(context_permissions::permission_id.asc())
        .select(context_permissions::allowed)
        .first::<bool>(conn)
        .optional()
}
