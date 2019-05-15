use crate::{
    auth,
    db::SolDbConn,
    models::{Reading, ReadingInsert, Sensor, SensorInsert, Token, User},
    result::{Error, Result},
};
use chrono::NaiveDateTime;
use rocket::{get, http::RawStr, post, request::FromFormValue};
use rocket_contrib::json::Json;

mod res;
use self::res::{Data, Message};

#[derive(Deserialize)]
pub struct Register {
    email: String,
    password: String,
}

#[post("/users/new", format = "application/json", data = "<user>")]
pub fn add_user(user: Json<Register>, conn: SolDbConn) -> Result<Message> {
    User::insert(user.0.email, user.0.password, &conn)
        .map(|_| Message::new("successfully created user"))
}

#[get("/users/all")]
pub fn get_users(conn: SolDbConn) -> Result<Data> {
    User::all(&conn).map(|users| Data::new("found all users", json!({ "users": users })))
}

pub struct UnixEpochTime(NaiveDateTime);

impl<'v> FromFormValue<'v> for UnixEpochTime {
    type Error = &'v RawStr;
    fn from_form_value(form_value: &'v RawStr) -> std::result::Result<Self, Self::Error> {
        let unix = i64::from_form_value(form_value)?;
        Ok(UnixEpochTime(NaiveDateTime::from_timestamp(unix, 0)))
    }
}

impl<'de> serde::Deserialize<'de> for UnixEpochTime {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let unix = i64::deserialize(deserializer)?;
        let datetime = NaiveDateTime::from_timestamp(unix, 0);
        Ok(UnixEpochTime(datetime))
    }
}

#[get("/sensor/<id>/readings?<start>&<end>")]
pub fn get_readings(
    id: i32,
    start: UnixEpochTime,
    end: UnixEpochTime,
    conn: SolDbConn,
) -> Result<Data> {
    Reading::find_for_sensor_in_time_range(id, start.0, end.0, &conn).map(|readings| {
        Data::new(
            "found all readings for sensor in range",
            json!({ "readings": readings }),
        )
    })
}

#[derive(Deserialize)]
pub struct SensorHardwareId {
    hardware_id: i64,
}

#[post("/sensor_token", format = "application/json", data = "<sensor_hw_id>")]
pub fn get_sensor_token(
    auth: auth::UserToken,
    conn: SolDbConn,
    sensor_hw_id: Json<SensorHardwareId>,
) -> Result<Data> {
    let user = auth.user();
    let hardware_id = sensor_hw_id.0.hardware_id;
    let sensor = Sensor::find_by_hardware_id(hardware_id, &conn)?;
    if user.id == sensor.owner_id {
        let token = Token::new_sensor_token(sensor);
        Token::insert(&token, &conn)
            .map(|_count| Data::new("got sensor token", json!({"token": token.token})))
    } else {
        Err(Error::NotSensorOwner)
    }
}

#[allow(non_snake_case)]
#[derive(Deserialize)]
pub struct CreateReading {
    timestamp: UnixEpochTime,
    peak_power_mW: f32,
    peak_current_mA: f32,
    peak_voltage_V: f32,
    temp_celsius: f32,
    batt_V: f32,
}

#[post("/add_reading", format = "application/json", data = "<reading>")]
pub fn add_reading(
    auth: auth::SensorToken,
    conn: SolDbConn,
    reading: Json<CreateReading>,
) -> Result<Message> {
    let reading = ReadingInsert {
        sensor_id: auth.sensor().id,
        timestamp: reading.0.timestamp.0,
        peak_power_mW: reading.0.peak_power_mW,
        peak_current_mA: reading.0.peak_current_mA,
        peak_voltage_V: reading.0.peak_voltage_V,
        temp_celsius: reading.0.temp_celsius,
        batt_V: reading.0.batt_V,
    };
    Reading::insert(&reading, &conn).map(|_| Message::new("successfully added reading"))
}

#[post("/add_readings", format = "application/json", data = "<readings>")]
pub fn add_readings(
    auth: auth::SensorToken,
    conn: SolDbConn,
    readings: Json<Vec<CreateReading>>,
) -> Result<Message> {
    let sensor_id = auth.sensor().id;
    let readings = readings
        .0
        .iter()
        .map(|r| ReadingInsert {
            sensor_id,
            timestamp: r.timestamp.0,
            peak_power_mW: r.peak_power_mW,
            peak_current_mA: r.peak_current_mA,
            peak_voltage_V: r.peak_voltage_V,
            temp_celsius: r.temp_celsius,
            batt_V: r.batt_V,
        })
        .collect();
    Reading::insert_many(&readings, &conn).map(|_| Message::new("successfully added readings"))
}

#[post("/token")]
pub fn get_token(auth: auth::Basic, conn: SolDbConn) -> Result<Data> {
    let user = auth.user();
    let token = Token::new_user_token(&user);
    Token::insert(&token, &conn)
        .map(|_count| Data::new("got user token", json!({"token": token.token})))
}

#[derive(Serialize, Deserialize)]
pub struct CreateSensor {
    hardware_id: i64,
}

#[post("/add_sensor", format = "application/json", data = "<data>")]
pub fn add_sensor(
    auth: auth::UserToken,
    data: Json<CreateSensor>,
    conn: SolDbConn,
) -> Result<Message> {
    let sensor = SensorInsert {
        owner_id: auth.user().id,
        hardware_id: data.hardware_id,
    };
    Sensor::insert(&sensor, &conn).map(|_| Message::new("successfully added sensor"))
}
