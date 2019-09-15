use rocket::local::Client;

pub fn test_client() -> Client {
    let db_uri = "";
    crate::db::run_migrations(db_uri);
    let rocket = crate::rocket(db_uri);
    Client::new(rocket).expect("created test client")
}
