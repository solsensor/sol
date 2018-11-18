table! {
    readings (id) {
        id -> Integer,
        sensor_id -> Integer,
        timestamp -> Integer,
        peak_power_mW -> Float,
        peak_current_mA -> Float,
        peak_voltage_V -> Float,
        temp_celsius -> Float,
        batt_V -> Float,
    }
}

table! {
    sensors (id) {
        id -> Integer,
        owner_id -> Integer,
        hardware_id -> BigInt,
        active -> Bool,
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
        password -> Text,
    }
}

joinable!(sensors -> users (owner_id));
joinable!(tokens -> users (user_id));

allow_tables_to_appear_in_same_query!(readings, sensors, tokens, users,);
