use diesel::table;

table! {
    rooms(room_id) {
        room_id -> Integer,
        m_room_id -> Text,
        context_id -> Nullable<Integer>
    }
}
