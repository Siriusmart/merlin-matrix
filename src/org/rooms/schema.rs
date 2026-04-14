use diesel::table;

table! {
    rooms(room_id) {
        room_id -> Integer,
        m_room_id -> Text,
        m_room_homeserver -> Text,
        context_id -> Nullable<Integer>
    }
}
