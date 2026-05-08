use diesel::{
    RunQueryDsl, Selectable,
    prelude::{Identifiable, Insertable, Queryable},
    sqlite::Sqlite,
};

use crate::org::{DatabaseConnection, permissions::permissions};

#[derive(DieselNewType, Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub struct PermissionId(i32);

#[derive(Selectable, Queryable, Insertable, Identifiable)]
#[diesel(check_for_backend(Sqlite))]
#[diesel(table_name = super::schema::permissions)]
#[diesel(primary_key(permission_id))]
pub struct Permission {
    permission_id: PermissionId,
    qualifier: String,
}

#[derive(Insertable)]
#[diesel(table_name = super::schema::permissions)]
#[diesel(check_for_backend(Sqlite))]
struct NewPermission {
    qualifier: String,
}

impl Permission {
    /// insert into table if not exist
    pub fn ensure_exists(
        conn: &mut DatabaseConnection,
        qualifier: String,
    ) -> Result<(), diesel::result::Error> {
        diesel::insert_into(permissions::table)
            .values(NewPermission { qualifier })
            .on_conflict_do_nothing()
            .execute(conn)?;

        Ok(())
    }
}
