use diesel::insert_into;
use diesel::prelude::*;
use std::error::Error;
use super::schema::{tokens,users};

#[derive(Insertable, Serialize, Deserialize, FromForm)]
#[table_name = "users"]
pub struct UserInsert {
    pub id: Option<i32>,
    pub email: String,
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
    pub fn find(token: &String, conn: &SqliteConnection) -> Result<TokenQuery, impl Error> {
        use super::schema::tokens::dsl::{tokens as all_tokens, token as token_token};
        all_tokens.filter(token_token.eq(token)).first(conn)
    }

    pub fn insert(token: &TokenInsert, conn: &SqliteConnection) -> Result<usize, impl Error> {
        use super::schema::tokens::{table as tokens_table};
        insert_into(tokens_table).values(token).execute(conn)
    }
}
