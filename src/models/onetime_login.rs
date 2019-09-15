use crate::{result::Result, schema::onetime_logins, util};
use chrono::NaiveDateTime;
use diesel::{insert_into, prelude::*, Insertable, Queryable};

#[derive(Insertable, Serialize)]
#[table_name = "onetime_logins"]
pub struct OnetimeLoginInsert {
    pub token: String,
    pub user_id: i32,
}

#[derive(Serialize, Queryable, Debug)]
pub struct OnetimeLogin {
    pub token: String,
    pub user_id: i32,
    pub created: NaiveDateTime,
    pub expires: NaiveDateTime,
}

pub fn create(user_id: i32, conn: &SqliteConnection) -> Result<String> {
    let token = util::token::rand_str();
    let login = OnetimeLoginInsert { token, user_id };

    insert_into(onetime_logins::table)
        .values(&login)
        .execute(conn)?;

    Ok(login.token)
}

pub fn delete(tok: &String, conn: &SqliteConnection) -> Result<()> {
    use crate::schema::onetime_logins::dsl::{onetime_logins, token};
    diesel::delete(onetime_logins.filter(token.eq(tok))).execute(conn)?;
    Ok(())
}

pub fn find(tok: &String, conn: &SqliteConnection) -> Result<OnetimeLogin> {
    use crate::schema::onetime_logins::dsl::{onetime_logins, token};
    let cred = onetime_logins.filter(token.eq(tok)).first(conn)?;
    Ok(cred)
}
