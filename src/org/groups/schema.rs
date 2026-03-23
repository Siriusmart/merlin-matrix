use diesel::table;

table! {
    groups(id) {
        id -> Integer,
        display -> Text,
        owner_name -> Text,
        owner_homeserver -> Text,
        admin_group_id -> Nullable<Integer>
    }
}
