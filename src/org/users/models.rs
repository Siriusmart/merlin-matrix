use std::error::Error;

use diesel::{
    ExpressionMethods, OptionalExtension, RunQueryDsl, Selectable,
    prelude::{Identifiable, Insertable, Queryable},
    query_dsl::methods::FilterDsl,
    sqlite::Sqlite,
};

use crate::org::{DatabaseConnection, users::schema::users};

#[derive(DieselNewType, Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub struct UserId(i32);

/// A user is added to database on demand when it is used in
/// another database table
#[derive(Selectable, Queryable, Insertable, Identifiable, Hash, PartialEq, Eq)]
#[diesel(check_for_backend(Sqlite))]
#[diesel(table_name = super::schema::users)]
#[diesel(primary_key(user_id))]
pub struct User {
    user_id: UserId,
    m_user_id: String,
    m_user_homeserver: String,
}

#[derive(Insertable)]
#[diesel(table_name = users)]
#[diesel(check_for_backend(Sqlite))]
struct NewUser {
    m_user_id: String,
    m_user_homeserver: String,
}

impl User {
    pub fn id(&self) -> UserId {
        self.user_id
    }

    pub fn m_id(&self) -> &str {
        &self.m_user_id
    }

    pub fn m_homeserver(&self) -> &str {
        &self.m_user_homeserver
    }
}

impl User {
    pub fn get_with_id(
        conn: &mut DatabaseConnection,
        user_id: UserId,
    ) -> Result<Option<User>, Box<dyn Error>> {
        Ok(users::table
            .filter(users::user_id.eq(user_id))
            .first(conn)
            .optional()?)
    }

    pub fn get(
        conn: &mut DatabaseConnection,
        m_user_id: &str,
        m_user_homeserver: &str,
    ) -> Result<Option<User>, Box<dyn Error>> {
        Ok(users::table
            .filter(users::m_user_id.eq(m_user_id))
            .filter(users::m_user_homeserver.eq(m_user_homeserver))
            .first(conn)
            .optional()?)
    }

    pub fn get_or_create(
        conn: &mut DatabaseConnection,
        m_user_id: String,
        m_user_homeserver: String,
    ) -> Result<User, Box<dyn Error>> {
        Self::ensure_created(conn, m_user_id.clone(), m_user_homeserver.clone())?;

        Ok(users::table
            .filter(users::m_user_id.eq(m_user_id))
            .filter(users::m_user_homeserver.eq(m_user_homeserver))
            .first(conn)?)
    }

    /// create if not already exist, otherwise do nothing
    pub fn ensure_created(
        conn: &mut DatabaseConnection,
        m_user_id: String,
        m_user_homeserver: String,
    ) -> Result<(), Box<dyn Error>> {
        diesel::insert_into(users::table)
            .values(&NewUser {
                m_user_id,
                m_user_homeserver,
            })
            .on_conflict((users::m_user_id, users::m_user_homeserver))
            .do_nothing()
            .execute(conn)?;
        Ok(())
    }
}
