use diesel::{
    Selectable,
    prelude::{Associations, Identifiable, Insertable, Queryable},
    sqlite::Sqlite,
};

use crate::org::contexts::ContextId;

#[derive(DieselNewType, Debug, Hash, PartialEq, Eq)]
pub struct RoomId(i32);

#[derive(Selectable, Queryable, Insertable, Identifiable, Associations)]
#[diesel(check_for_backend(Sqlite))]
#[diesel(table_name = super::schema::rooms)]
#[diesel(primary_key(room_id))]
#[diesel(belongs_to(crate::org::contexts::Context, foreign_key = context_id))]
pub struct Room {
    room_id: RoomId,
    matrix_room_id: String,
    matrix_room_homeserver: String,
    context_id: Option<ContextId>,
}
