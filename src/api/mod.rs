use crate::{
    auth,
    db::SolDbConn,
    models::{
        Energy, Reading, ReadingInsert, ReadingQueryUnix, Sensor, SensorInsert, Token, User,
        UserQuery,
    },
    result::{Error, Result},
};
use chrono::NaiveDateTime;
use git_version::git_version;
use rocket::{get, http::RawStr, post, request::FromFormValue};
use rocket_contrib::json::Json;

mod res;
use self::res::Data;

pub(crate) mod result;
use self::result::Result as ApiResult;

#[derive(Deserialize)]
pub struct Register {
    email: String,
    password: String,
}

#[derive(Serialize)]
pub struct AddUserResponse {}

#[post("/users/new", format = "application/json", data = "<user>")]
pub fn add_user(user: Json<Register>, conn: SolDbConn) -> ApiResult<Json<AddUserResponse>> {
    User::insert(user.0.email, user.0.password, &conn)?;
    Ok(Json(AddUserResponse {}))
}

#[derive(Serialize)]
pub struct GetUsersResponse {
    pub users: Vec<UserQuery>,
}

#[get("/users/all")]
pub fn get_users(
    conn: SolDbConn,
    admin: Result<auth::AdminToken>,
) -> ApiResult<Json<GetUsersResponse>> {
    admin?;
    let res = User::all(&conn)
        .map(|users| GetUsersResponse { users })
        .map(Json)?;
    Ok(res)
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

#[derive(Serialize)]
pub struct GetEnergyStatsResponse {
    pub stats: Vec<Energy>,
}

#[get("/sensor/<id>/energy_stats")]
pub fn get_energy_stats(id: i32, conn: SolDbConn) -> ApiResult<Json<GetEnergyStatsResponse>> {
    let res = Sensor::energy_stats(id, &conn)
        .map(|stats| GetEnergyStatsResponse { stats })
        .map(Json)?;
    Ok(res)
}

#[get("/sensor/<id>/readings?<start>&<end>&<unixtime>")]
pub fn get_readings(
    id: i32,
    start: UnixEpochTime,
    end: UnixEpochTime,
    conn: SolDbConn,
    unixtime: Option<bool>,
) -> ApiResult<Data> {
    let res =
        Reading::find_for_sensor_in_time_range(id, start.0, end.0, &conn).map(|readings| {
            let obj = match unixtime {
                Some(true) => {
                    let rs: Vec<ReadingQueryUnix> =
                        readings.into_iter().map(ReadingQueryUnix::from).collect();
                    json!({ "readings": rs })
                }
                _ => json!({ "readings": readings }),
            };
            Data::new("found all readings for sensor in range", obj)
        })?;
    Ok(res)
}

#[derive(Deserialize)]
pub struct SensorHardwareId {
    hardware_id: i64,
}

#[derive(Serialize)]
pub struct GetSensorTokenResponse {
    pub token: String,
}

#[post("/sensor_token", format = "application/json", data = "<sensor_hw_id>")]
pub fn get_sensor_token(
    auth: auth::UserToken,
    conn: SolDbConn,
    sensor_hw_id: Json<SensorHardwareId>,
) -> ApiResult<Json<GetSensorTokenResponse>> {
    let user = auth.user();
    let hardware_id = sensor_hw_id.0.hardware_id;
    let sensor = Sensor::find_by_hardware_id(hardware_id, &conn)?;
    if user.id == sensor.owner_id {
        let token = Token::new_sensor_token(sensor);
        Token::insert(&token, &conn)?;
        Ok(Json(GetSensorTokenResponse { token: token.token }))
    } else {
        Err(Error::NotSensorOwner.into())
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

#[derive(Serialize)]
pub struct AddReadingResponse {}

#[post("/add_reading", format = "application/json", data = "<reading>")]
pub fn add_reading(
    auth: auth::SensorToken,
    conn: SolDbConn,
    reading: Json<CreateReading>,
) -> ApiResult<Json<AddReadingResponse>> {
    let reading = ReadingInsert {
        sensor_id: auth.sensor().id,
        timestamp: reading.0.timestamp.0,
        peak_power_mW: reading.0.peak_power_mW,
        peak_current_mA: reading.0.peak_current_mA,
        peak_voltage_V: reading.0.peak_voltage_V,
        temp_celsius: reading.0.temp_celsius,
        batt_V: reading.0.batt_V,
    };
    Reading::insert(&reading, &conn)?;
    Ok(Json(AddReadingResponse {}))
}

#[derive(Serialize)]
pub struct AddReadingsResponse {}

#[post("/add_readings", format = "application/json", data = "<readings>")]
pub fn add_readings(
    auth: auth::SensorToken,
    conn: SolDbConn,
    readings: Json<Vec<CreateReading>>,
) -> ApiResult<Json<AddReadingsResponse>> {
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
    Reading::insert_many(&readings, &conn)?;
    Ok(Json(AddReadingsResponse {}))
}

#[derive(Serialize, Deserialize, PartialEq)]
pub struct GetTokenResponse {
    pub token: String,
}

#[post("/token")]
pub fn get_token(auth: Result<auth::Basic>, conn: SolDbConn) -> ApiResult<Json<GetTokenResponse>> {
    let user = auth?.user();
    let token = Token::new_user_token(&user);
    let res =
        Token::insert(&token, &conn).map(|_count| Json(GetTokenResponse { token: token.token }))?;
    Ok(res)
}

#[derive(Serialize, Deserialize)]
pub struct CreateSensor {
    hardware_id: i64,
}

#[derive(Serialize)]
pub struct AddSensorResponse {}

#[post("/add_sensor", format = "application/json", data = "<data>")]
pub fn add_sensor(
    auth: Result<auth::UserToken>,
    data: Json<CreateSensor>,
    conn: SolDbConn,
) -> ApiResult<Json<AddSensorResponse>> {
    let auth = auth?;
    let sensor = SensorInsert {
        owner_id: auth.user().id,
        hardware_id: data.hardware_id,
    };
    Sensor::insert(&sensor, &conn)?;
    Ok(Json(AddSensorResponse {}))
}

#[derive(Serialize)]
pub struct GetVersionResponse {
    pub version: String,
}

const GIT_VERSION: &str = git_version!();

#[get("/version")]
pub fn get_version() -> ApiResult<Json<GetVersionResponse>> {
    Ok(Json(GetVersionResponse {
        version: GIT_VERSION.to_string(),
    }))
}
