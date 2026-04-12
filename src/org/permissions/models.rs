use diesel::{
    Selectable,
    prelude::{Identifiable, Insertable, Queryable},
    sqlite::Sqlite,
};

#[derive(DieselNewType, Debug, Hash, PartialEq, Eq)]
pub struct PermissionId(i32);

#[derive(Selectable, Queryable, Insertable, Identifiable)]
#[diesel(check_for_backend(Sqlite))]
#[diesel(table_name = super::schema::permissions)]
#[diesel(primary_key(permission_id))]
pub struct Permission {
    permission_id: PermissionId,
    qualifier: String,
}
