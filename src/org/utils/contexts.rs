use diesel::{
    ExpressionMethods, JoinOnDsl, NullableExpressionMethods, OptionalExtension, QueryDsl,
    RunQueryDsl, SqliteExpressionMethods, prelude::Queryable,
};
use tracing::*;

use crate::org::{
    DatabaseConnection,
    context_permissions::{ContextPermissionPriority, context_permissions},
    contexts::{Context, ContextId, contexts},
    group_users::group_users::{self, table},
    groups::{GroupId, groups},
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
        .order_by(context_permissions::priority.desc())
        .then_order_by(context_permissions::permission_id.asc())
        .select(context_permissions::allowed)
        .first::<bool>(conn)
        .optional()
}

/// get context of a room
#[instrument(skip_all)]
pub fn context_of_room(
    conn: &mut DatabaseConnection,
    m_room_id: &str,
) -> Result<Option<Context>, diesel::result::Error> {
    trace!("m_room_id={m_room_id}");
    rooms::table
        .inner_join(contexts::table.on(rooms::context_id.eq(contexts::context_id.nullable())))
        .filter(rooms::m_room_id.eq(m_room_id))
        .select(contexts::all_columns)
        .first(conn)
        .optional()
}

#[instrument(skip_all)]
pub fn count_rooms_with_context(
    conn: &mut DatabaseConnection,
    context_id: ContextId,
) -> Result<i64, diesel::result::Error> {
    rooms::table
        .filter(rooms::context_id.eq(context_id))
        .count()
        .get_result(conn)
}

#[instrument(skip_all)]
pub fn list_user_context_owned_s(
    conn: &mut DatabaseConnection,
    m_user_id: &str,
    m_user_homeserver: &str,
) -> Result<Vec<String>, diesel::result::Error> {
    trace!("m_user_id={m_user_id:?} m_user_homeserver={m_user_homeserver:?}");

    users::table
        .inner_join(contexts::table.on(users::user_id.eq(contexts::owner_id)))
        .filter(users::m_user_id.eq(m_user_id))
        .filter(users::m_user_homeserver.eq(m_user_homeserver))
        .select(contexts::name)
        .get_results(conn)
}

#[instrument(skip_all)]
pub fn list_user_context_admin_s(
    conn: &mut DatabaseConnection,
    m_user_id: &str,
    m_user_homeserver: &str,
) -> Result<Vec<String>, diesel::result::Error> {
    trace!("m_user_id={m_user_id:?} m_user_homeserver={m_user_homeserver:?}");

    users::table
        .inner_join(group_users::table.on(users::user_id.eq(group_users::user_id)))
        .inner_join(
            contexts::table.on(group_users::group_id
                .nullable()
                .eq(contexts::admin_group_id)),
        )
        .filter(users::m_user_id.eq(m_user_id))
        .filter(users::m_user_homeserver.eq(m_user_homeserver))
        .select(contexts::name)
        .get_results(conn)
}

#[derive(Queryable)]
pub struct ContextPermissionEntry {
    pub qualifier: String,
    pub allowed: bool,
    pub group_id: GroupId,
    pub group_name: String,
    pub priority: ContextPermissionPriority,
}

pub fn permissions_of_context(
    conn: &mut DatabaseConnection,
    context_id: ContextId,
) -> Result<Vec<ContextPermissionEntry>, diesel::result::Error> {
    context_permissions::table
        .inner_join(
            permissions::table
                .on(permissions::permission_id.eq(context_permissions::permission_id)),
        )
        .inner_join(contexts::table.on(contexts::context_id.eq(context_permissions::context_id)))
        .inner_join(groups::table.on(groups::group_id.eq(context_permissions::group_id)))
        .filter(context_permissions::context_id.eq(context_id))
        .select((
            permissions::qualifier,
            context_permissions::allowed,
            groups::group_id,
            groups::name,
            context_permissions::priority,
        ))
        .order_by(context_permissions::permission_id)
        .then_order_by(context_permissions::priority)
        .then_order_by(context_permissions::group_id)
        .get_results(conn)
}
