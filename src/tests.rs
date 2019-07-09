use rocket::{http::Status, local::Client};
use std::fs;
use insta::assert_snapshot_matches;

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

macro_rules! snap {
    (id: $id:ident, method: $method:ident, path: $path:expr,) => (
        #[test]
        fn $id() {
            let client = setup();
            let mut res = client.$method($path).dispatch();
            let contents = res.body_string().expect("no body content");
            assert_eq!(res.status(), Status::Ok);
            assert_snapshot_matches!(stringify!($id), contents);
        }
    )
}

snap!{
    id: get_users,
    method: get,
    path: "/api/users/all",
}
