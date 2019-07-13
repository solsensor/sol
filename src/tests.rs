use rocket::{http::Status, local::Client};
use std::{io, fs };
use insta::assert_snapshot_matches;

fn setup(db_path: &str) -> Client {
    fs::create_dir_all("target/testdbs/").expect("failed to create test db dir");
    fs::remove_file(db_path).or_else(|e| match e.kind() {
        io::ErrorKind::NotFound => Ok(()),
        _ => Err(e),
    }).expect("failed to remove existing test db");
    super::db::run_migrations(db_path);
    let rocket = super::rocket(super::rocket_config(db_path));
    Client::new(rocket).expect("created test client")
}

#[test]
fn test_simple_ok() {
    let client = setup("target/testdbs/simple_ok.sqlite");
    let res = client.get("/").dispatch();
    assert_eq!(res.status(), Status::Ok);
}

#[test]
fn test_simple_not_ok() {
    let client = setup("target/testdbs/simple_not_ok.sqlite");
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
