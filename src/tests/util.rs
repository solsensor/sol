use crate::util::token::rand_str;
use rocket::{
    http::{ContentType, Header, Status},
    local::{Client, LocalResponse},
};

#[macro_export]
macro_rules! json_string {
    ($json:tt) => {
        serde_json::to_string(&json!($json)).expect("could not stringify json")
    };
}

pub fn test_client() -> Client {
    let db_uri = format!("./target/testdbs/{}.db", rand_str());
    crate::db::run_migrations(&db_uri);
    let rocket = crate::rocket(&db_uri, true);
    Client::new(rocket).expect("created test client")
}

pub fn response_json_value(response: &mut LocalResponse) -> serde_json::Value {
    let body = response.body().expect("no body");
    serde_json::from_reader(body.into_inner()).expect("cannot parse response into json")
}

pub fn basic_auth_header(email: &str, password: &str) -> Header<'static> {
    let hash = base64::encode(&format!("{}:{}", email, password));
    Header::new("Authorization", format!("Basic {}", hash))
}

pub fn token_auth_header(token: &str) -> Header<'static> {
    // TODO also test capital bearer
    Header::new("Authorization", format!("bearer {}", token))
}

pub fn register(client: &Client, email: &str, pass: &str) {
    let mut res = client
        .post("/api/users/new")
        .header(ContentType::JSON)
        .body(json_string!({"email": email, "password": pass}))
        .dispatch();
    let _data = response_json_value(&mut res);
    assert_eq!(res.status(), Status::Ok);
}

pub fn get_token(client: &Client, email: &str, pass: &str) -> String {
    let mut res = client
        .post("/api/token")
        .header(ContentType::JSON)
        .header(basic_auth_header(email, pass))
        .dispatch();
    let data = response_json_value(&mut res);
    let token = data
        .get("token")
        .expect("must have a 'token' field")
        .as_str()
        .expect("value must be string");
    assert_eq!(res.status(), Status::Ok);
    token.to_string()
}

pub fn add_sensor(client: &Client, token: &str, hw_id: usize) {
    let mut res = client
        .post("/api/add_sensor")
        .header(ContentType::JSON)
        .header(token_auth_header(token))
        .body(json!({ "hardware_id": hw_id }).to_string())
        .dispatch();
    let _data = response_json_value(&mut res);
    assert_eq!(res.status(), Status::Ok);
}
