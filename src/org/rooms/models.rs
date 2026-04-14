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
    m_room_id: String,
    m_room_homeserver: String,
    context_id: Option<ContextId>,
}

#[derive(Insertable)]
#[diesel(table_name = rooms)]
#[diesel(check_for_backend(Sqlite))]
struct NewRoom {
    m_room_id: String,
    m_room_homeserver: String,
}

impl Room {
    pub fn get_or_create(
        pool: &DatabasePool,
        m_room_id: String,
        m_room_homeserver: String,
    ) -> Result<Room, Box<dyn Error>> {
        let new_room = NewRoom {
            m_room_id,
            m_room_homeserver,
        };

        let mut conn = pool.get().unwrap();

        diesel::insert_into(rooms::table)
            .values(&new_room)
            .on_conflict_do_nothing()
            .execute(&mut conn)?;

        Ok(rooms::table
            .filter(rooms::m_room_id.eq(new_room.m_room_id))
            .filter(rooms::m_room_homeserver.eq(new_room.m_room_homeserver))
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
