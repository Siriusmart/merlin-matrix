use diesel::table;

table! {
    groups(group_id) {
        group_id -> Integer,
        name -> Text,
        description -> Text,
        owner_id -> Integer,
        admin_group_id -> Nullable<Integer>,
    }
}
