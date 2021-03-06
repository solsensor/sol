pub mod onetime_login;

use crate::{
    result::{Error, Result},
    schema::{readings, sensors, tokens, users},
    util,
};
use chrono::{naive::serde::ts_seconds, NaiveDate, NaiveDateTime};
use diesel::{
    insert_into,
    prelude::*,
    sql_query,
    sql_types::{Date, Float},
    update, Insertable, Queryable,
};

#[allow(non_snake_case)]
#[derive(Insertable, Serialize, Deserialize)]
#[table_name = "readings"]
pub struct ReadingInsert {
    pub sensor_id: i32,
    pub timestamp: NaiveDateTime,
    pub peak_power_mW: f32,
    pub peak_current_mA: f32,
    pub peak_voltage_V: f32,
    pub temp_celsius: f32,
    pub batt_V: f32,
}

#[allow(non_snake_case)]
#[derive(Serialize, Queryable, Debug)]
pub struct ReadingQuery {
    pub id: i32,
    pub sensor_id: i32,
    pub timestamp: NaiveDateTime,
    pub peak_power_mW: f32,
    pub peak_current_mA: f32,
    pub peak_voltage_V: f32,
    pub temp_celsius: f32,
    pub batt_V: f32,
    pub created: NaiveDateTime,
}

#[allow(non_snake_case)]
#[derive(Serialize)]
pub struct ReadingQueryUnix {
    id: i32,
    sensor_id: i32,
    #[serde(with = "ts_seconds")]
    timestamp: NaiveDateTime,
    peak_power_mW: f32,
    peak_current_mA: f32,
    peak_voltage_V: f32,
    temp_celsius: f32,
    batt_V: f32,
    #[serde(with = "ts_seconds")]
    created: NaiveDateTime,
}

impl From<ReadingQuery> for ReadingQueryUnix {
    fn from(r: ReadingQuery) -> Self {
        ReadingQueryUnix {
            id: r.id,
            sensor_id: r.sensor_id,
            timestamp: r.timestamp,
            peak_power_mW: r.peak_power_mW,
            peak_current_mA: r.peak_current_mA,
            peak_voltage_V: r.peak_voltage_V,
            temp_celsius: r.temp_celsius,
            batt_V: r.batt_V,
            created: r.created,
        }
    }
}

pub struct Reading;

impl Reading {
    pub fn count(conn: &SqliteConnection) -> Result<i64> {
        use super::schema::readings::dsl::readings as all_readings;
        use diesel::dsl::count_star;

        let count: i64 = all_readings.select(count_star()).first(conn)?;
        Ok(count)
    }

    pub fn insert(reading: &ReadingInsert, conn: &SqliteConnection) -> Result<usize> {
        use super::schema::readings::table as readings_table;
        insert_into(readings_table)
            .values(reading)
            .execute(conn)
            .map_err(|e| e.into())
    }

    pub fn insert_many(readings: &Vec<ReadingInsert>, conn: &SqliteConnection) -> Result<usize> {
        use super::schema::readings::table as readings_table;
        insert_into(readings_table)
            .values(readings)
            .execute(conn)
            .map_err(|e| e.into())
    }

    pub fn find_for_sensor(sensor_id: i32, conn: &SqliteConnection) -> Result<Vec<ReadingQuery>> {
        use super::schema::readings::dsl::{
            readings as all_readings, sensor_id as reading_sensor_id, timestamp,
        };
        all_readings
            .filter(reading_sensor_id.eq(sensor_id))
            .order(timestamp.desc())
            .load(conn)
            .map_err(|e| e.into())
    }

    pub fn find_for_sensor_in_time_range(
        sensor_id: i32,
        start: NaiveDateTime,
        end: NaiveDateTime,
        conn: &SqliteConnection,
    ) -> Result<Vec<ReadingQuery>> {
        use super::schema::readings::dsl::{
            readings as all_readings, sensor_id as reading_sensor_id,
            timestamp as reading_timestamp,
        };
        all_readings
            .filter(reading_sensor_id.eq(sensor_id))
            .filter(reading_timestamp.gt(start))
            .filter(reading_timestamp.lt(end))
            .load(conn)
            .map_err(|e| e.into())
    }
}

#[allow(non_snake_case)]
#[derive(Serialize, QueryableByName, Clone, Copy)]
pub struct Energy {
    #[sql_type = "Date"]
    pub date: NaiveDate,
    #[sql_type = "Float"]
    pub equiv_kWh: f32,
    #[sql_type = "Float"]
    pub cap_factor: f32,
    #[sql_type = "Float"]
    pub dollars_saved: f32,
    #[sql_type = "Float"]
    pub co2_saved: f32,
}

#[derive(Insertable, Serialize, Clone)]
#[table_name = "users"]
pub struct UserInsert {
    pub email: String,
    pub pwd_hash: String,
}

#[derive(Serialize, Queryable, Debug)]
pub struct UserQuery {
    pub id: i32,
    pub email: String,
    pub pwd_hash: String,
    pub superuser: bool,
}

pub struct User;

impl User {
    fn hash_password(pw: String) -> String {
        let bytes = argon2rs::argon2i_simple(&pw, "salty salt");
        base64::encode(&bytes)
    }

    pub fn all(conn: &SqliteConnection) -> Result<Vec<UserQuery>> {
        use super::schema::users::dsl::users as all_users;
        all_users.load::<UserQuery>(conn).map_err(|e| e.into())
    }

    pub fn by_email(email: &String, conn: &SqliteConnection) -> Result<UserQuery> {
        use super::schema::users::dsl::{email as user_email, users as all_users};
        all_users
            .filter(user_email.eq(email))
            .first(conn)
            .map_err(|e| e.into())
    }

    pub fn by_id(id: i32, conn: &SqliteConnection) -> Result<UserQuery> {
        use super::schema::users::dsl::{id as user_id, users as all_users};
        all_users
            .filter(user_id.eq(id))
            .first(conn)
            .map_err(|e| e.into())
    }

    pub fn by_token(token: &String, conn: &SqliteConnection) -> Result<UserQuery> {
        match Token::find(token, conn)?.user_id {
            Some(id) => Self::by_id(id, conn),
            None => Err(Error::WrongTokenType),
        }
    }

    pub fn by_onetime(tok: &String, conn: &SqliteConnection) -> Result<UserQuery> {
        let cred = onetime_login::find(tok, conn)?;
        User::by_id(cred.user_id, conn)
    }

    pub fn verify_password(
        email: &String,
        password: &String,
        conn: &SqliteConnection,
    ) -> Result<UserQuery> {
        Self::by_email(email, conn).and_then(|user| {
            if user.pwd_hash == Self::hash_password(password.to_string()) {
                Ok(user)
            } else {
                Err(Error::IncorrectPassword)
            }
        })
    }

    pub fn update(id: i32, email: &str, conn: &SqliteConnection) -> Result<()> {
        update(users::table.find(id))
            .set(users::email.eq(email))
            .execute(conn)
            .map(|_| ())?;
        Ok(())
    }

    pub fn update_password(user_id: i32, pwd: String, conn: &SqliteConnection) -> Result<()> {
        let hash = Self::hash_password(pwd);
        update(users::table.find(user_id))
            .set(users::pwd_hash.eq(hash))
            .execute(conn)
            .map(|_| ())?;
        Ok(())
    }

    pub fn insert(email: String, password: String, conn: &SqliteConnection) -> Result<usize> {
        use super::schema::users::table as users_table;
        let user = UserInsert {
            email,
            pwd_hash: Self::hash_password(password),
        };
        insert_into(users_table)
            .values(user)
            .execute(conn)
            .map_err(|e| e.into())
    }
}

#[derive(Serialize, Deserialize)]
pub enum TokenType {
    User,
    Sensor,
}

impl TokenType {
    pub fn from_string(s: String) -> TokenType {
        match s.as_str() {
            "user" => TokenType::User,
            "sensor" => TokenType::Sensor,
            _ => panic!(format!("invalid string {} for token", s)),
        }
    }

    pub fn get_string(&self) -> String {
        let s = match self {
            TokenType::User => "user",
            TokenType::Sensor => "sensor",
        };
        s.to_string()
    }
}

#[derive(Insertable, Serialize, Deserialize)]
#[table_name = "tokens"]
pub struct TokenInsert {
    pub token: String,
    pub type_: String,
    pub user_id: Option<i32>,
    pub sensor_id: Option<i32>,
}

#[derive(Serialize, Queryable, Debug)]
pub struct TokenQuery {
    pub token: String,
    pub type_: String,
    pub user_id: Option<i32>,
    pub sensor_id: Option<i32>,
}

pub struct Token;

impl Token {
    pub fn new_user_token(user: &UserQuery) -> TokenInsert {
        TokenInsert {
            token: format!("user-{}", util::token::rand_str()),
            type_: TokenType::User.get_string(),
            user_id: Some(user.id),
            sensor_id: None,
        }
    }

    pub fn new_sensor_token(sensor: SensorQuery) -> TokenInsert {
        TokenInsert {
            token: format!("sensor-{}", util::token::rand_str()),
            type_: TokenType::Sensor.get_string(),
            user_id: None,
            sensor_id: Some(sensor.id),
        }
    }

    pub fn find(token: &String, conn: &SqliteConnection) -> Result<TokenQuery> {
        use super::schema::tokens::dsl::{token as token_token, tokens as all_tokens};
        all_tokens
            .filter(token_token.eq(token))
            .first(conn)
            .map_err(|e| e.into())
    }

    pub fn insert(token: &TokenInsert, conn: &SqliteConnection) -> Result<usize> {
        use super::schema::tokens::table as tokens_table;
        insert_into(tokens_table)
            .values(token)
            .execute(conn)
            .map_err(|e| e.into())
    }
}

#[derive(Insertable, Serialize, Deserialize)]
#[table_name = "sensors"]
pub struct SensorInsert {
    pub owner_id: i32,
    pub hardware_id: i64,
}

#[derive(Serialize, Deserialize, Queryable, Debug)]
pub struct SensorQuery {
    pub id: i32,
    pub owner_id: i32,
    pub hardware_id: i64,
    pub active: bool,
    pub name: Option<String>,
    pub description: Option<String>,
}

pub struct Sensor;

impl Sensor {
    pub fn count(conn: &SqliteConnection) -> Result<i64> {
        use super::schema::sensors::dsl::sensors as all_sensors;
        use diesel::dsl::count_star;

        let count: i64 = all_sensors.select(count_star()).first(conn)?;
        Ok(count)
    }

    pub fn find(id: i32, conn: &SqliteConnection) -> Result<SensorQuery> {
        use super::schema::sensors::dsl::{id as sensor_id, sensors as all_sensors};
        all_sensors
            .filter(sensor_id.eq(id))
            .first(conn)
            .map_err(|e| e.into())
    }

    pub fn update(
        id: i32,
        name: String,
        description: String,
        conn: &SqliteConnection,
    ) -> Result<()> {
        update(sensors::table.find(id))
            .set((sensors::name.eq(name), sensors::description.eq(description)))
            .execute(conn)
            .map(|_| ())?;
        Ok(())
    }

    pub fn find_by_hardware_id(hardware_id: i64, conn: &SqliteConnection) -> Result<SensorQuery> {
        use super::schema::sensors::dsl::{
            active as sensor_active, hardware_id as sensor_hardware_id, sensors as all_sensors,
        };
        all_sensors
            .filter(sensor_hardware_id.eq(hardware_id))
            .filter(sensor_active.eq(true))
            .first(conn)
            .map_err(|e| e.into())
    }

    pub fn find_for_user(user_id: i32, conn: &SqliteConnection) -> Result<Vec<SensorQuery>> {
        use super::schema::sensors::dsl::{
            active as sensor_active, owner_id as sensor_owner_id, sensors as all_sensors,
        };
        all_sensors
            .filter(sensor_owner_id.eq(user_id))
            .filter(sensor_active.eq(true))
            .load(conn)
            .map_err(|e| e.into())
    }

    pub fn insert(sensor: &SensorInsert, conn: &SqliteConnection) -> Result<usize> {
        use super::schema::sensors::{
            dsl::{
                active as sensor_active, hardware_id as sensor_hardware_id, sensors as all_sensors,
            },
            table as sensors_table,
        };
        use diesel::dsl::count_star;

        let count: i64 = all_sensors
            .select(count_star())
            .filter(sensor_hardware_id.eq(sensor.hardware_id))
            .filter(sensor_active.eq(true))
            .first(conn)?;

        if count != 0 {
            return Err(Error::DuplicateHardwareId(sensor.hardware_id));
        }

        insert_into(sensors_table)
            .values(sensor)
            .execute(conn)
            .map_err(|e| e.into())
    }

    pub fn deactivate(conn: &SqliteConnection, id: i32) -> Result<()> {
        update(sensors::table.find(id))
            .set(sensors::active.eq(false))
            .execute(conn)
            .map(|_| ())?;
        Ok(())
    }

    pub fn energy_stats(sensor_id: i32, conn: &SqliteConnection) -> Result<Vec<Energy>> {
        let query = format!(
"
with
 src as (select * from readings where sensor_id = {sensor_id} and timestamp > date('now', '-{lookback_days} days')),
 wins as (select peak_power_mW as pa, timestamp as ta, lead(peak_power_mW) over win as pb, lead(timestamp) over win as tb from src window win as (order by timestamp asc)),
 filtered as (select * from wins where tb is not null),
 calcs as (select ta as ts, min(pa, pb) as p0, abs(pa-pb) as dp, (julianday(tb)-julianday(ta))*(24) as dt from filtered),
 areas as (select ts, p0*dt + dp*dt*0.5 as mWh from calcs),
 by_day as (select date(ts) as date, sum(mWh) as sol_mWh from areas group by date),
 stats1 as (select *, sol_mWh*7895 as equiv_mWh, sol_mWh/(570*24) as cap_factor from by_day),
 stats2 as (select *, equiv_mWh/(1000*1000) as equiv_kWh from stats1),
 stats3 as (select *, equiv_kWh*0.12 as dollars_saved, equiv_kWh*1.6 as co2_saved from stats2)
select * from stats3;
",
            sensor_id = sensor_id,
            lookback_days = 10
        );
        let res: Vec<Energy> = sql_query(query).load(conn)?;
        Ok(res)
    }
}
