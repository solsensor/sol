use rocket::{http::Status, local::Client};

#[test]
fn test_simple_ok() {
    let rocket = super::rocket();
    let client = Client::new(rocket).expect("valid rocket instance");
    let res = client.get("/").dispatch();

    assert_eq!(res.status(), Status::Ok);
}

#[test]
fn test_simple_not_ok() {
    let rocket = super::rocket();
    let client = Client::new(rocket).expect("valid rocket instance");
    let res = client.get("/invalid_endpoint").dispatch();

    assert_eq!(res.status(), Status::NotFound);
}
