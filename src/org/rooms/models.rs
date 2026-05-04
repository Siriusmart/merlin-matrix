use std::error::Error;

use diesel::{
    ExpressionMethods, RunQueryDsl, Selectable,
    prelude::{Associations, Identifiable, Insertable, Queryable},
    query_dsl::methods::FilterDsl,
    sqlite::Sqlite,
};

use crate::org::{DatabaseConnection, contexts::ContextId, rooms::schema::rooms};

#[derive(DieselNewType, Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub struct RoomId(i32);

/// a room that has information stored about it
#[derive(Selectable, Queryable, Insertable, Identifiable, Associations)]
#[diesel(check_for_backend(Sqlite))]
#[diesel(table_name = rooms)]
#[diesel(primary_key(room_id))]
#[diesel(belongs_to(crate::org::contexts::Context, foreign_key = context_id))]
pub struct Room {
    room_id: RoomId,
    m_room_id: String,
    context_id: Option<ContextId>,
}

#[derive(Insertable)]
#[diesel(table_name = rooms)]
#[diesel(check_for_backend(Sqlite))]
struct NewRoom {
    m_room_id: String,
}

impl Room {
    pub fn id(&self) -> RoomId {
        self.room_id
    }

    pub fn context_id(&self) -> Option<ContextId> {
        self.context_id
    }
}

impl Room {
    pub fn get_or_create(
        conn: &mut DatabaseConnection,
        m_room_id: String,
    ) -> Result<Room, Box<dyn Error>> {
        let new_room = NewRoom { m_room_id };

        diesel::insert_into(rooms::table)
            .values(&new_room)
            .on_conflict_do_nothing()
            .execute(conn)?;

        Ok(rooms::table
            .filter(rooms::m_room_id.eq(new_room.m_room_id))
            .first(conn)?)
    }

    pub fn set_context_id(
        &mut self,
        conn: &mut DatabaseConnection,
        context_id: Option<ContextId>,
    ) -> Result<(), Box<dyn Error>> {
        diesel::update(&*self)
            .set(rooms::context_id.eq(&context_id))
            .execute(conn)?;

        self.context_id = context_id;

        Ok(())
    }
}
