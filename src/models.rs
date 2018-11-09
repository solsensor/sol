use super::schema::{readings, sensors, tokens, users};
use diesel::insert_into;
use diesel::prelude::*;
use rand::Rng;
use std::error::Error;
use std::iter;

#[derive(Insertable, Serialize, Deserialize)]
#[table_name = "readings"]
pub struct ReadingInsert {
    pub id: Option<i32>,
    pub voltage: f32,
    pub sensor_id: i32,
}

pub struct Reading;

impl Reading {
    pub fn insert(reading: &ReadingInsert, conn: &SqliteConnection) -> Result<usize, impl Error> {
        use super::schema::readings::table as readings_table;
        insert_into(readings_table).values(reading).execute(conn)
    }
}

#[derive(Insertable, Serialize, Deserialize, FromForm)]
#[table_name = "users"]
pub struct UserInsert {
    pub id: Option<i32>,
    pub email: String,
    pub password: String,
}

#[derive(Serialize, Queryable, Debug)]
pub struct UserQuery {
    pub id: i32,
    pub email: String,
    pub password: String,
}

pub struct User;

impl User {
    pub fn all(conn: &SqliteConnection) -> Result<Vec<UserQuery>, impl Error> {
        use super::schema::users::dsl::users as all_users;
        all_users.load::<UserQuery>(conn)
    }

    pub fn by_email(email: &String, conn: &SqliteConnection) -> Result<UserQuery, impl Error> {
        use super::schema::users::dsl::{email as user_email, users as all_users};
        all_users.filter(user_email.eq(email)).first(conn)
    }

    pub fn by_id(id: i32, conn: &SqliteConnection) -> Result<UserQuery, impl Error> {
        use super::schema::users::dsl::{id as user_id, users as all_users};
        all_users.filter(user_id.eq(id)).first(conn)
    }

    pub fn verify_password(
        email: &String,
        password: &String,
        conn: &SqliteConnection,
    ) -> Result<UserQuery, String> {
        match Self::by_email(email, conn) {
            Ok(user) => {
                if &user.password == password {
                    Ok(user)
                } else {
                    Err(format!("incorrect password"))
                }
            }
            Err(err) => Err(format!("failed to get user: {}", err.to_string())),
        }
    }

    pub fn insert(user: &UserInsert, conn: &SqliteConnection) -> Result<usize, impl Error> {
        use super::schema::users::table as users_table;
        insert_into(users_table).values(user).execute(conn)
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
    fn rand_str() -> String {
        let mut rng = rand::thread_rng();
        iter::repeat(())
            .map(|()| rng.sample(rand::distributions::Alphanumeric))
            .take(64)
            .collect()
    }

    pub fn new_user_token(user: UserQuery) -> TokenInsert {
        TokenInsert {
            token: Self::rand_str(),
            type_: TokenType::User.get_string(),
            user_id: Some(user.id),
            sensor_id: None,
        }
    }

    pub fn new_sensor_token(sensor: SensorQuery) -> TokenInsert {
        TokenInsert {
            token: Self::rand_str(),
            type_: TokenType::Sensor.get_string(),
            user_id: None,
            sensor_id: Some(sensor.id),
        }
    }

    pub fn find(token: &String, conn: &SqliteConnection) -> Result<TokenQuery, impl Error> {
        use super::schema::tokens::dsl::{token as token_token, tokens as all_tokens};
        all_tokens.filter(token_token.eq(token)).first(conn)
    }

    pub fn insert(token: &TokenInsert, conn: &SqliteConnection) -> Result<usize, impl Error> {
        use super::schema::tokens::table as tokens_table;
        insert_into(tokens_table).values(token).execute(conn)
    }
}

#[derive(Insertable, Serialize, Deserialize)]
#[table_name = "sensors"]
pub struct SensorInsert {
    pub owner_id: i32,
    pub hardware_id: i32,
}

#[derive(Serialize, Deserialize, Queryable, Debug)]
pub struct SensorQuery {
    pub id: i32,
    pub owner_id: i32,
    pub hardware_id: i32,
}

pub struct Sensor;

impl Sensor {
    pub fn find(id: i32, conn: &SqliteConnection) -> Result<SensorQuery, impl Error> {
        use super::schema::sensors::dsl::{id as sensor_id, sensors as all_sensors};
        all_sensors.filter(sensor_id.eq(id)).first(conn)
    }

    pub fn insert(sensor: &SensorInsert, conn: &SqliteConnection) -> Result<usize, impl Error> {
        use super::schema::sensors::table as sensors_table;
        insert_into(sensors_table).values(sensor).execute(conn)
    }
}
