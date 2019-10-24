use crate::tests::util::test_client;
use rocket::http::{ContentType, Status};

macro_rules! assert_body_json {
    ($res:ident, $js:tt) => {
        let contents = $res.body_string().expect("no body");
        let json = json!($js);
        assert_eq!(contents, json.to_string());
    };
}

macro_rules! assert_res_json {
    ($res:ident, $status:ident, $js:tt) => {
        assert_body_json!($res, $js);
        assert_eq!($res.status(), Status::$status);
    };
}

mod create_user {
    use super::*;

    #[test]
    fn get_users_unauthorized() {
        let client = test_client();
        let req = client.get("/api/users/all");
        let mut res = req.dispatch();
        assert_res_json!(res, Unauthorized, {
            "status": "failure",
            "message": "not authorized to access /api/users/all",
        });
    }

    #[test]
    fn simple() {
        let client = test_client();
        let req = client
            .post("/api/users/new")
            .header(ContentType::JSON)
            .body(json!({"email": "newuser@gmail.com", "password": "mypassword"}).to_string());

        let mut res = req.dispatch();
        assert_res_json!(res, Ok, {
            "status": "success",
            "message": "successfully created user",
        });
    }
}
