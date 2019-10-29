use crate::{
    json_string,
    tests::util::{
        add_sensor, basic_auth_header, get_token, register, response_json_value, test_client,
        token_auth_header,
    },
};
use rocket::http::{ContentType, Status};

#[test]
fn get_users_unauthorized() {
    let client = test_client();
    let req = client.get("/api/users/all");
    let mut res = req.dispatch();
    let data = response_json_value(&mut res);
    let error = data
        .get("error")
        .expect("must have 'error' field")
        .as_str()
        .expect("value must be string");
    assert_eq!(error, "ApiError(missing token)");
}

#[test]
fn get_token_no_auth_header() {
    let client = test_client();
    let mut res = client.post("/api/token").dispatch();
    let data = response_json_value(&mut res);
    let error = data
        .get("error")
        .expect("must have 'error' field")
        .as_str()
        .expect("value must be string");
    assert_eq!(error, "ApiError(missing basic auth header)");
}

#[test]
fn create_user_get_token() {
    let client = test_client();
    register(&client, "newuser@gmail.com", "mypassword");
    let tok = get_token(&client, "newuser@gmail.com", "mypassword");
    assert_eq!(&tok[..5], "user-");
}

#[test]
fn create_user_duplicate_email() {
    let client = test_client();
    register(&client, "newuser@gmail.com", "mypassword");
    let mut res = client
        .post("/api/users/new")
        .header(ContentType::JSON)
        .body(json_string!({"email": "newuser@gmail.com", "password": "otherpassword"}))
        .dispatch();
    let data = response_json_value(&mut res);
    let error = data
        .get("error")
        .expect("response should have 'error' field")
        .as_str()
        .expect("should be str");
    assert_eq!(
        error,
        "ApiError(user with email 'newuser@gmail.com' already exists)"
    );
    assert_eq!(res.status(), Status::BadRequest);
}

#[test]
fn add_one_sensor() {
    let client = test_client();
    register(&client, "newuser@gmail.com", "mypassword");
    let tok = get_token(&client, "newuser@gmail.com", "mypassword");
    add_sensor(&client, &tok, 12);
}

#[test]
fn add_sensor_duplicate_hardware_id() {
    let client = test_client();
    register(&client, "newuser@gmail.com", "mypassword");
    let tok = get_token(&client, "newuser@gmail.com", "mypassword");
    add_sensor(&client, &tok, 12);

    let mut res = client
        .post("/api/add_sensor")
        .header(ContentType::JSON)
        .header(token_auth_header(&tok))
        .body(json!({"hardware_id": 12}).to_string())
        .dispatch();
    let data = response_json_value(&mut res);
    let error = data
        .get("error")
        .expect("response should have 'error' field")
        .as_str()
        .expect("should be str");
    assert_eq!(
        error,
        "ApiError(active sensor with hardware id 12 already exists)"
    );
    assert_eq!(res.status(), Status::BadRequest);
}
