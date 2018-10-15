use diesel::insert_into;
use diesel::prelude::*;
use std::error::Error;
use std::iter;
use rand::Rng;
use super::schema::{tokens,users,sensors};

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

pub struct User();

impl User {
    pub fn all(conn: &SqliteConnection) -> Result<Vec<UserQuery>, impl Error> {
        use super::schema::users::dsl::{users as all_users};
        all_users.load::<UserQuery>(conn)
    }

    pub fn by_email(email: &String, conn: &SqliteConnection) -> Result<UserQuery, impl Error> {
        use super::schema::users::dsl::{users as all_users, email as user_email};
        all_users.filter(user_email.eq(email)).first(conn)
    }

    pub fn by_id(id: i32, conn: &SqliteConnection) -> Result<UserQuery, impl Error> {
        use super::schema::users::dsl::{users as all_users, id as user_id};
        all_users.filter(user_id.eq(id)).first(conn)
    }

    pub fn verify_password(email: &String, password: &String, conn: &SqliteConnection) -> Result<UserQuery, String> {
        match Self::by_email(email, conn) {
            Ok(user) => if &user.password == password {
                    Ok(user)
                } else {
                    Err(format!("incorrect password"))
                },
            Err(err) => Err(format!("failed to get user: {}", err.to_string())),
        }
    }

    pub fn insert(user: &UserInsert, conn: &SqliteConnection) -> Result<usize, impl Error> {
        use super::schema::users::{table as users_table};
        insert_into(users_table).values(user).execute(conn)
    }
}

#[derive(Insertable, Serialize, Deserialize)]
#[table_name = "tokens"]
pub struct TokenInsert {
    pub token: String,
    pub user_id: i32,
}

#[derive(Serialize, Queryable, Debug)]
pub struct TokenQuery {
    pub token: String,
    pub user_id: i32,
}

pub struct Token();

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
            user_id: user.id,
        }
    }

    pub fn find(token: &String, conn: &SqliteConnection) -> Result<TokenQuery, impl Error> {
        use super::schema::tokens::dsl::{tokens as all_tokens, token as token_token};
        all_tokens.filter(token_token.eq(token)).first(conn)
    }

    pub fn insert(token: &TokenInsert, conn: &SqliteConnection) -> Result<usize, impl Error> {
        use super::schema::tokens::{table as tokens_table};
        insert_into(tokens_table).values(token).execute(conn)
    }
}

#[derive(Insertable, Serialize, Deserialize)]
#[table_name = "sensors"]
pub struct SensorInsert {
    pub owner_id: i32,
    pub hardware_id: i32,
}

#[derive(Serialize, Queryable, Debug)]
pub struct SensorQuery {
    pub id: i32,
    pub owner_id: i32,
    pub hardware_id: i32,
}

pub struct Sensor();

impl Sensor {
    pub fn find(id: i32, conn: &SqliteConnection) -> Result<SensorQuery, impl Error> {
        use super::schema::sensors::dsl::{sensors as all_sensors, id as sensor_id};
        all_sensors.filter(sensor_id.eq(id)).first(conn)
    }

    pub fn insert(sensor: &SensorInsert, conn: &SqliteConnection) -> Result<usize, impl Error> {
        use super::schema::sensors::{table as sensors_table};
        insert_into(sensors_table).values(sensor).execute(conn)
    }
}
