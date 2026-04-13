use diesel::table;

table! {
    context_permissions(context_id, permission_id) {
        context_id -> Integer,
        group_id -> Integer,
        permission_id -> Integer,
        priority -> Integer,
        allowed -> Bool
    }
}
