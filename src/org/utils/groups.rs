use std::error::Error;

use diesel::{
    ExpressionMethods, JoinOnDsl, NullableExpressionMethods, OptionalExtension, QueryDsl,
    RunQueryDsl, SqliteExpressionMethods,
};
use tracing::*;

use crate::org::{
    DatabaseConnection,
    group_users::{GroupUser, group_users},
    groups::{GroupId, groups},
    users::{User, UserId, users},
};

/// add user to group, return Ok(true) is user is previously not in group and now added to group
#[instrument(skip_all)]
pub fn add_user_to_group(
    conn: &mut DatabaseConnection,
    user_id: UserId,
    group_id: GroupId,
) -> Result<bool, Box<dyn Error>> {
    trace!("user_id={user_id:?} group_id={group_id:?}");
    let inserted = diesel::insert_into(group_users::table)
        .values(&GroupUser::new(user_id, group_id))
        .on_conflict((group_users::user_id, group_users::group_id))
        .do_nothing()
        .execute(conn)?;

    Ok(inserted != 0)
}

#[instrument(skip_all)]
pub fn remove_user_from_group(
    conn: &mut DatabaseConnection,
    user_id: UserId,
    group_id: GroupId,
) -> Result<bool, Box<dyn Error>> {
    trace!("user_id={user_id:?} group_id={group_id:?}");
    let removed = diesel::delete(group_users::table)
        .filter(group_users::user_id.eq(user_id))
        .filter(group_users::group_id.eq(group_id))
        .execute(conn)?;

    Ok(removed != 0)
}

/// test if user is in a group
#[instrument(skip_all)]
pub fn user_id_in_group_id(
    conn: &mut DatabaseConnection,
    user_id: UserId,
    group_id: GroupId,
) -> Result<bool, Box<dyn Error>> {
    trace!("user_id={user_id:?} group_id={group_id:?}");
    Ok(group_users::table
        .filter(group_users::user_id.eq(user_id))
        .filter(group_users::group_id.eq(group_id))
        .first::<GroupUser>(conn)
        .optional()?
        .is_some())
}

/// return list of group names the users is in
#[instrument(skip_all)]
pub fn list_user_groups_s(
    conn: &mut DatabaseConnection,
    m_user_id: &str,
    m_user_homeserver: &str,
) -> Result<Vec<String>, diesel::result::Error> {
    trace!("m_user_id={m_user_id:?} m_user_homeserver={m_user_homeserver:?}");
    users::table
        .inner_join(group_users::table.on(group_users::user_id.eq(users::user_id)))
        .filter(users::m_user_id.eq(m_user_id))
        .filter(users::m_user_homeserver.eq(m_user_homeserver))
        .inner_join(groups::table.on(group_users::group_id.eq(groups::group_id)))
        .distinct()
        .select(groups::name)
        .get_results(conn)
}

/// return list of group names the users is admin of
#[instrument(skip_all)]
pub fn list_user_groups_admin_s(
    conn: &mut DatabaseConnection,
    m_user_id: &str,
    m_user_homeserver: &str,
) -> Result<Vec<String>, diesel::result::Error> {
    trace!("m_user_id={m_user_id:?} m_user_homeserver={m_user_homeserver:?}");

    let admin_groups = diesel::alias!(groups as admin_groups);

    users::table
        .inner_join(group_users::table.on(group_users::user_id.eq(users::user_id)))
        .filter(users::m_user_id.eq(m_user_id))
        .filter(users::m_user_homeserver.eq(m_user_homeserver))
        .inner_join(admin_groups.on(group_users::group_id.eq(admin_groups.field(groups::group_id))))
        .inner_join(
            groups::table
                .on(groups::admin_group_id.is(admin_groups.field(groups::group_id).nullable())),
        )
        .distinct()
        .select(groups::name)
        .get_results(conn)
}

/// return list of group names the users is admin of
#[instrument(skip_all)]
pub fn list_user_groups_owned_s(
    conn: &mut DatabaseConnection,
    m_user_id: &str,
    m_user_homeserver: &str,
) -> Result<Vec<String>, diesel::result::Error> {
    trace!("m_user_id={m_user_id:?} m_user_homeserver={m_user_homeserver:?}");
    users::table
        .inner_join(groups::table.on(groups::owner_id.eq(users::user_id)))
        .filter(users::m_user_id.eq(m_user_id))
        .filter(users::m_user_homeserver.eq(m_user_homeserver))
        .distinct()
        .select(groups::name)
        .get_results(conn)
}

/// list users in group
#[instrument(skip_all)]
pub fn list_group_members(
    conn: &mut DatabaseConnection,
    group_id: GroupId,
    limit: Option<i64>,
) -> Result<Vec<User>, diesel::result::Error> {
    trace!("group_id={group_id:?} limit={limit:?}");
    users::table
        .inner_join(group_users::table.on(group_users::user_id.eq(users::user_id)))
        .filter(group_users::group_id.eq(group_id))
        .limit(limit.unwrap_or(i64::MAX))
        .select(users::all_columns)
        .get_results(conn)
}

/// list users in group
#[instrument(skip_all)]
pub fn count_group_members(
    conn: &mut DatabaseConnection,
    group_id: GroupId,
) -> Result<i64, diesel::result::Error> {
    trace!("group_id={group_id:?}");
    users::table
        .inner_join(group_users::table.on(group_users::user_id.eq(users::user_id)))
        .filter(group_users::group_id.eq(group_id))
        .count()
        .get_result(conn)
}
