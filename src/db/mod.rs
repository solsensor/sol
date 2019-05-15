use diesel::Connection;
use diesel_migrations::embed_migrations;

#[database("sqlite_sol")]
pub struct SolDbConn(diesel::SqliteConnection);

embed_migrations!("./migrations");

pub fn run_migrations() {
    let conn = diesel::SqliteConnection::establish("./sol.sqlite").expect("error connecting to db");
    embedded_migrations::run(&conn).expect("failed to run migrations");
}
