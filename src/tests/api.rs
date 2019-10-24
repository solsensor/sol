use crate::tests::util::test_client;
use rocket::http::{ContentType, Header, Status};

macro_rules! assert_body_json {
    ($res:ident, $js:tt) => {
        let contents = $res.body_string().expect("no body");
        let json = json!($js);
        assert_eq!(contents, json.to_string());
    };
}

macro_rules! assert_res_json {
    ($res:ident, $status:expr, $js:tt) => {
        assert_body_json!($res, $js);
        assert_eq!($res.status(), $status);
    };
}

fn basic_auth_header(email: &str, password: &str) -> Header<'static> {
    let hash = base64::encode(&format!("{}:{}", email, password));
    Header::new("Authorization", format!("Basic {}", hash))
}

mod user {
    use super::*;

    #[test]
    fn get_users_unauthorized() {
        let client = test_client();
        let req = client.get("/api/users/all");
        let mut res = req.dispatch();
        assert_res_json!(res, Status::BadRequest, {
            "status": "failure",
            "message": "ApiError(missing token)",
        });
    }

    #[test]
    fn get_token_no_auth_header() {
        let client = test_client();
        let mut res = client.post("/api/token").dispatch();
        assert_res_json!(res, Status::BadRequest, {
            "status": "failure",
            "message": "ApiError(missing basic auth header)",
        });
    }

    #[test]
    fn create_and_get_token() {
        let client = test_client();

        let mut res = client
            .post("/api/users/new")
            .header(ContentType::JSON)
            .body(json!({"email": "newuser@gmail.com", "password": "mypassword"}).to_string())
            .dispatch();
        assert_res_json!(res, Status::Ok, {
            "status": "success",
            "message": "successfully created user",
        });

        let mut res = client
            .post("/api/token")
            .header(ContentType::JSON)
            .header(basic_auth_header("newuser@gmail.com", "mypassword"))
            .body(json!({"email": "newuser@gmail.com", "password": "mypassword"}).to_string())
            .dispatch();
        assert_res_json!(res, Status::Ok, {
            "status": "success",
            "message": "got user token",
        });
    }
}
