use crate::tests::util::test_client;
use insta::assert_snapshot_matches;
use rocket::http::Status;

macro_rules! snap {
    (id: $id:ident, method: $method:ident, path: $path:expr,) => {
        #[test]
        fn $id() {
            let client = test_client();
            let mut res = client.$method($path).dispatch();
            let contents = res.body_string().expect("no body content");
            assert_eq!(res.status(), Status::Ok);
            assert_snapshot_matches!(stringify!($id), contents);
        }
    };
}

snap! {
    id: get_users,
    method: get,
    path: "/api/users/all",
}
