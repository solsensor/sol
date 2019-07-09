use rocket::{http::Status, local::Client};
use std::fs;
use serde_json::{ json, Value as JsonValue };

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

#[test]
fn test_get_users() {
    let client = setup();
    let mut res = client.get("/api/users/all").dispatch();
    let contents = res.body_string().expect("no body content");
    let actual: JsonValue = serde_json::from_str(&contents).expect("failed to parse json");
    let expected = json!({
        "data": {
            "users": [],
        },
        "status": "success",
        "message": "found all users",
    });

    assert_eq!(res.status(), Status::Ok);
    assert_eq!(actual, expected);
}
