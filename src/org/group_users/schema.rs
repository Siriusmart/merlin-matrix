use diesel::table;

table! {
    group_users(user_id, group_id) {
        user_id -> Integer,
        group_id -> Integer
    }
}
