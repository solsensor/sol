table! {
    onetime_logins (token) {
        token -> Text,
        user_id -> Integer,
        created -> Timestamp,
        expires -> Timestamp,
    }
}

table! {
    readings (id) {
        id -> Integer,
        sensor_id -> Integer,
        timestamp -> Timestamp,
        peak_power_mW -> Float,
        peak_current_mA -> Float,
        peak_voltage_V -> Float,
        temp_celsius -> Float,
        batt_V -> Float,
        created -> Timestamp,
    }
}

table! {
    sensors (id) {
        id -> Integer,
        owner_id -> Integer,
        hardware_id -> BigInt,
        active -> Bool,
        name -> Nullable<Text>,
        description -> Nullable<Text>,
    }
}

table! {
    tokens (token) {
        token -> Text,
        #[sql_name = "type"]
        type_ -> Text,
        user_id -> Nullable<Integer>,
        sensor_id -> Nullable<Integer>,
    }
}

table! {
    users (id) {
        id -> Integer,
        email -> Text,
        pwd_hash -> Text,
        superuser -> Bool,
    }
}

joinable!(onetime_logins -> users (user_id));
joinable!(sensors -> users (owner_id));
joinable!(tokens -> users (user_id));

allow_tables_to_appear_in_same_query!(onetime_logins, readings, sensors, tokens, users,);
