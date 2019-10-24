use crate::util::token::rand_str;
use rocket::local::Client;

pub fn test_client() -> Client {
    let db_uri = format!("./target/testdbs/{}.db", rand_str());
    crate::db::run_migrations(&db_uri);
    let rocket = crate::rocket(&db_uri, true);
    Client::new(rocket).expect("created test client")
}
