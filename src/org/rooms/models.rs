use std::error::Error;

use diesel::{
    ExpressionMethods, RunQueryDsl, Selectable,
    prelude::{Associations, Identifiable, Insertable, Queryable},
    query_dsl::methods::FilterDsl,
    sqlite::Sqlite,
};

use crate::org::{DatabasePool, contexts::ContextId, rooms::schema::rooms};

#[derive(DieselNewType, Debug, Hash, PartialEq, Eq)]
pub struct RoomId(i32);

/// a room that has information stored about it
#[derive(Selectable, Queryable, Insertable, Identifiable, Associations)]
#[diesel(check_for_backend(Sqlite))]
#[diesel(table_name = rooms)]
#[diesel(primary_key(room_id))]
#[diesel(belongs_to(crate::org::contexts::Context, foreign_key = context_id))]
pub struct Room {
    room_id: RoomId,
    matrix_room_id: String,
    matrix_room_homeserver: String,
    context_id: Option<ContextId>,
}

#[derive(Insertable)]
#[diesel(table_name = rooms)]
#[diesel(check_for_backend(Sqlite))]
pub struct NewRoom {
    matrix_room_id: String,
    matrix_room_homeserver: String,
}

impl Room {
    pub fn get_or_create(
        pool: &DatabasePool,
        matrix_room_id: String,
        matrix_room_homeserver: String,
    ) -> Result<Room, Box<dyn Error>> {
        let new_room = NewRoom {
            matrix_room_id,
            matrix_room_homeserver,
        };

        let mut conn = pool.get().unwrap();

        diesel::insert_into(rooms::table)
            .values(&new_room)
            .on_conflict_do_nothing()
            .execute(&mut conn)?;

        Ok(rooms::table
            .filter(rooms::matrix_room_id.eq(new_room.matrix_room_id))
            .filter(rooms::matrix_room_homeserver.eq(new_room.matrix_room_homeserver))
            .first(&mut conn)?)
    }

    pub fn set_context_id(
        &mut self,
        pool: &DatabasePool,
        context_id: Option<ContextId>,
    ) -> Result<(), Box<dyn Error>> {
        let mut conn = pool.get().unwrap();

        diesel::update(&*self)
            .set(rooms::context_id.eq(&context_id))
            .execute(&mut conn)?;

        self.context_id = context_id;

        Ok(())
    }
}
