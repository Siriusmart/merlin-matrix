use diesel::table;

table! {
    rooms(room_id) {
        room_id -> Integer,
        matrix_room_id -> Text,
        matrix_room_homeserver -> Text,
        context_id -> Nullable<Integer>
    }
}
