use diesel::table;

table! {
    permissions(permission_id) {
        permission_id -> Integer,
        qualifier -> Text,
    }
}
