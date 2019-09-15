use diesel::{Connection, SqliteConnection};
use diesel_migrations::embed_migrations;

embed_migrations!("./migrations");

#[database("sqlite_sol")]
pub struct SolDbConn(SqliteConnection);

pub fn run_migrations(uri: &str) {
    let conn = SqliteConnection::establish(uri).expect("error connecting to db");
    embedded_migrations::run(&conn).expect("failed to run migrations");
}
