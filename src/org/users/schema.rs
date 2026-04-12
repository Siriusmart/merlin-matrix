use diesel::table;

table! {
    users(user_id) {
        user_id -> Integer,
        name -> Text,
        homeserver -> Text
    }
}
