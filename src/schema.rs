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

joinable!(tokens -> users (user_id));

allow_tables_to_appear_in_same_query!(
    tokens,
    users,
);
