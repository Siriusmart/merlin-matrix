use std::error::Error;

use diesel::{
    ExpressionMethods, RunQueryDsl, Selectable,
    prelude::{Identifiable, Insertable, Queryable},
    query_dsl::methods::FilterDsl,
    sqlite::Sqlite,
};

use crate::org::{DatabasePool, users::schema::users};

#[derive(DieselNewType, Debug, Hash, PartialEq, Eq)]
pub struct UserId(i32);

/// A user is added to database on demand when it is used in
/// another database table
#[derive(Selectable, Queryable, Insertable, Identifiable)]
#[diesel(check_for_backend(Sqlite))]
#[diesel(table_name = super::schema::users)]
#[diesel(primary_key(user_id))]
pub struct User {
    user_id: UserId,
    name: String,
    homeserver: String,
}

#[derive(Insertable)]
#[diesel(table_name = users)]
#[diesel(check_for_backend(Sqlite))]
pub struct NewUser {
    name: String,
    homeserver: String,
}

impl User {
    pub fn get_or_create(
        pool: &DatabasePool,
        name: String,
        homeserver: String,
    ) -> Result<User, Box<dyn Error>> {
        let new_user = NewUser { name, homeserver };

        let mut conn = pool.get().unwrap();

        diesel::insert_into(users::table)
            .values(&new_user)
            .on_conflict_do_nothing()
            .execute(&mut conn)?;

        Ok(users::table
            .filter(users::name.eq(new_user.name))
            .filter(users::homeserver.eq(new_user.homeserver))
            .first(&mut conn)?)
    }
}
