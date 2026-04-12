use diesel::prelude::Identifiable;

#[derive(DieselNewType, Debug, Hash, PartialEq, Eq)]
pub struct UserId(i32);

#[derive(Identifiable)]
#[diesel(check_for_backend(Sqlite))]
#[diesel(table_name = super::schema::users)]
#[diesel(primary_key(user_id))]
pub struct User {
    user_id: UserId,
    name: String,
    homeserver: String,
}
