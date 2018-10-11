table! {
    sensors (id) {
        id -> Integer,
        owner_id -> Integer,
        hardware_id -> Integer,
    }
}

table! {
    tokens (token) {
        token -> Text,
        user_id -> Integer,
    }
}

table! {
    users (id) {
        id -> Integer,
        email -> Text,
        password -> Text,
    }
}

joinable!(sensors -> users (owner_id));
joinable!(tokens -> users (user_id));

allow_tables_to_appear_in_same_query!(
    sensors,
    tokens,
    users,
);
