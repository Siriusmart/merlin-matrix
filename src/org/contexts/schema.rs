use diesel::table;

table! {
    contexts(context_id) {
        context_id -> Integer,
        name -> Text,
        description -> Text,
        owner_id -> Integer,
        admin_group_id -> Nullable<Integer>
    }
}
