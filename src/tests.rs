use rocket::{http::Status, local::Client};
use std::fs;

fn setup() -> Client {
    fs::remove_file("sol.sqlite");
    super::db::run_migrations();
    let rocket = super::rocket();
    Client::new(rocket).expect("created test client")
}

#[test]
fn test_simple_ok() {
    let client = setup();
    let res = client.get("/").dispatch();
    assert_eq!(res.status(), Status::Ok);
}

#[test]
fn test_simple_not_ok() {
    let client = setup();
    let res = client.get("/invalid_endpoint").dispatch();
    assert_eq!(res.status(), Status::NotFound);
}
