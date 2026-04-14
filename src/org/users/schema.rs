use diesel::table;

table! {
    users(user_id) {
        user_id -> Integer,
        m_user_id -> Text,
        m_user_homeserver -> Text
    }
}
