use crate::tests::util::test_client;
use rocket::http::Status;

#[test]
fn test_simple_ok() {
    let client = test_client();
    let res = client.get("/").dispatch();
    assert_eq!(res.status(), Status::Ok);
}

#[test]
fn test_simple_not_ok() {
    let client = test_client();
    let res = client.get("/invalid_endpoint").dispatch();
    assert_eq!(res.status(), Status::NotFound);
}
