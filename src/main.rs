#![feature(plugin)]
#![feature(custom_attribute)]
#![feature(custom_derive)]
#![plugin(rocket_codegen)]
#![allow(proc_macro_derive_resolution_fallback)]

extern crate rand;
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate serde_derive;

mod db;
mod models;
mod schema;

use models::{
    Reading, ReadingInsert, ReadingQuery, Sensor, SensorInsert, SensorQuery, Token, TokenQuery,
    TokenType, User, UserInsert, UserQuery,
};
use rocket::{
    data::{self, Data, FromData},
    http::Status,
    request::{Form, FromRequest},
    response::{NamedFile, Redirect},
    Outcome, Request, Rocket,
};
use rocket_contrib::{Json, Template, Value};

use std::path::{Path, PathBuf};

#[derive(Serialize)]
struct TemplateCtx {
    title: String,
    users: Option<Vec<UserQuery>>,
    user: Option<UserQuery>,
    sensors: Option<Vec<SensorQuery>>,
    sensor: Option<SensorQuery>,
    readings: Option<Vec<ReadingQuery>>,
}

#[get("/")]
fn index() -> Template {
    let ctx = TemplateCtx {
        title: String::from("Home"),
        users: None,
        user: None,
        sensors: None,
        sensor: None,
        readings: None,
    };
    Template::render("index", &ctx)
}

#[get("/users")]
fn users(conn: db::Conn) -> Template {
    let users = User::all(conn.handler()).ok();
    let ctx = TemplateCtx {
        title: String::from("Users"),
        user: None,
        users,
        sensors: None,
        sensor: None,
        readings: None,
    };
    Template::render("users", &ctx)
}

#[get("/user/<email>")]
fn user(email: String, conn: db::Conn) -> Template {
    let user = User::by_email(&email, conn.handler()).unwrap();
    let sensors = Sensor::find_for_user(user.id, conn.handler()).ok();
    let ctx = TemplateCtx {
        title: email,
        users: None,
        user: Some(user),
        sensors,
        sensor: None,
        readings: None,
    };
    Template::render("user", &ctx)
}

#[get("/sensor/<id>")]
fn sensor(id: i32, conn: db::Conn) -> Template {
    let sensor = Sensor::find(id, conn.handler()).unwrap();
    let readings = Reading::find_for_sensor(sensor.id, conn.handler()).ok();
    let ctx = TemplateCtx {
        title: format!("sensor {}", id),
        users: None,
        user: None,
        sensors: None,
        sensor: Some(sensor),
        readings,
    };
    Template::render("sensor", &ctx)
}

#[get("/login")]
fn login() -> Template {
    let ctx = TemplateCtx {
        title: String::from("Login"),
        user: None,
        users: None,
        sensors: None,
        sensor: None,
        readings: None,
    };
    Template::render("login", &ctx)
}

#[post("/login", data = "<creds>")]
fn login_post(creds: Form<EmailPassword>, conn: db::Conn) -> Result<Redirect, String> {
    let creds = creds.get();
    let res = User::verify_password(&creds.email, &creds.password, conn.handler());
    match res {
        Ok(_user) => Ok(Redirect::to(&format!("/user/{}", creds.email))), // TODO set cookie or something
        Err(err) => Err(err),
    }
}

#[get("/register")]
fn register() -> Template {
    let ctx = TemplateCtx {
        title: String::from("Register"),
        user: None,
        users: None,
        sensors: None,
        sensor: None,
        readings: None,
    };
    Template::render("register", &ctx)
}

#[post("/register", data = "<form>")]
fn register_post(form: Form<UserInsert>, conn: db::Conn) -> Result<Redirect, String> {
    let user = form.get();
    let res = User::insert(user, conn.handler());
    match res {
        Ok(_count) => Ok(Redirect::to(&format!("/user/{}", user.email))),
        Err(err) => Err(err.to_string()),
    }
}

#[get("/<path..>")]
fn files(path: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new("static/").join(path)).ok()
}

#[post("/users/new", format = "application/json", data = "<user>")]
fn add_user(user: Json<UserInsert>, conn: db::Conn) -> Json<Value> {
    let res = User::insert(&user.0, conn.handler());
    match res {
        Ok(_count) => Json(json!({
            "status": "success",
        })),
        Err(err) => Json(json!({
            "status": "failed",
            "reason": err.to_string()
        })),
    }
}

#[get("/users/all")]
fn get_users(conn: db::Conn) -> String {
    format!(
        "all users: {:?}",
        User::all(conn.handler()).expect("failed to get all users")
    )
}

#[post(
    "/sensor_token",
    format = "application/json",
    data = "<sensor>"
)]
fn get_sensor_token(auth: UserTokenAuth, conn: db::Conn, sensor: Json<SensorQuery>) -> String {
    let user = auth.0;
    let sensor = sensor.0;
    if user.id == sensor.owner_id {
        let token = Token::new_sensor_token(sensor);
        let res = Token::insert(&token, conn.handler());
        match res {
            Ok(_count) => token.token,
            Err(err) => format!("failed: {}", err.to_string()),
        }
    } else {
        "user does not own sensor".to_string()
    }
}

#[derive(Serialize, Deserialize)]
struct CreateReading {
    voltage: f32,
}

#[post(
    "/add_reading",
    format = "application/json",
    data = "<reading>"
)]
fn add_reading(auth: SensorTokenAuth, conn: db::Conn, reading: Json<CreateReading>) -> String {
    let reading = ReadingInsert {
        id: None,
        voltage: reading.0.voltage,
        sensor_id: auth.0.id,
    };
    match Reading::insert(&reading, conn.handler()) {
        Ok(_) => format!("success!"),
        Err(err) => format!("err: {}", err.to_string()),
    }
}

#[post("/token", format = "application/json", data = "<auth>")]
fn get_token(auth: PasswordAuth, conn: db::Conn) -> String {
    let user = auth.0;
    let token = Token::new_user_token(user);
    let res = Token::insert(&token, conn.handler());
    match res {
        Ok(_count) => token.token,
        Err(err) => format!("failed: {}", err.to_string()),
    }
}

struct PasswordAuth(UserQuery);

#[derive(Deserialize, FromForm)]
struct EmailPassword {
    email: String,
    password: String,
}

impl FromData for PasswordAuth {
    type Error = String;
    fn from_data(req: &Request, data: Data) -> data::Outcome<Self, String> {
        let login: Json<EmailPassword> =
            Json::from_data(req, data).expect("failed to turn data into json");
        let login = login.into_inner();

        let conn: db::Conn = req.guard().expect("the request guard failed");
        let res = User::verify_password(&login.email, &login.password, conn.handler());

        match res {
            Ok(user) => Outcome::Success(PasswordAuth(user)),
            Err(err) => Outcome::Failure((Status::Unauthorized, err)),
        }
    }
}

struct TokenAuth(TokenQuery);

impl<'a, 'r> FromRequest<'a, 'r> for TokenAuth {
    type Error = String;
    fn from_request(req: &'a Request<'r>) -> Outcome<Self, (Status, String), ()> {
        let keys: Vec<_> = req.headers().get("Authorization").collect();
        if keys.len() != 1 {
            return Outcome::Failure((Status::Unauthorized, String::from("Missing Token")));
        }

        let words: Vec<String> = keys[0]
            .to_string()
            .split_whitespace()
            .map(String::from)
            .collect();
        if words.len() != 2 || words[0] != "bearer" {
            return Outcome::Failure((Status::Unauthorized, String::from("Malformed Token")));
        }

        let conn: db::Conn = req.guard().expect("req guard failed");
        let tok = Token::find(&words[1], conn.handler()).expect("could not find token");

        Outcome::Success(TokenAuth(tok))
    }
}

struct SensorTokenAuth(SensorQuery);
impl<'a, 'r> FromRequest<'a, 'r> for SensorTokenAuth {
    type Error = String;
    fn from_request(req: &'a Request<'r>) -> Outcome<Self, (Status, String), ()> {
        let token: TokenAuth = req.guard()?;
        let token = token.0;

        match TokenType::from_string(token.type_) {
            TokenType::User => Outcome::Failure((
                Status::Unauthorized,
                format!("expected sensor token, got user token"),
            )),
            TokenType::Sensor => {
                let conn: db::Conn = req.guard().expect("request guard failed");
                let sensor_id = token.sensor_id.expect("token had no sensor id");
                match Sensor::find(sensor_id, conn.handler()) {
                    Ok(sensor) => Outcome::Success(SensorTokenAuth(sensor)),
                    Err(_err) => Outcome::Failure((
                        Status::Unauthorized,
                        format!("could not find sensor for token"),
                    )),
                }
            }
        }
    }
}

struct UserTokenAuth(UserQuery);
impl<'a, 'r> FromRequest<'a, 'r> for UserTokenAuth {
    type Error = String;
    fn from_request(req: &'a Request<'r>) -> Outcome<Self, (Status, String), ()> {
        let token: TokenAuth = req.guard()?;
        let token = token.0;

        match TokenType::from_string(token.type_) {
            TokenType::Sensor => Outcome::Failure((
                Status::Unauthorized,
                format!("expected user token, got sensor token"),
            )),
            TokenType::User => {
                let conn: db::Conn = req.guard().expect("request guard failed");
                let user_id = token.user_id.expect("token had no user id");
                match User::by_id(user_id, conn.handler()) {
                    Ok(user) => Outcome::Success(UserTokenAuth(user)),
                    Err(_err) => Outcome::Failure((
                        Status::Unauthorized,
                        format!("could not find user for token"),
                    )),
                }
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
struct CreateSensor {
    hardware_id: i32,
}

#[post("/add_sensor", format = "application/json", data = "<data>")]
fn add_sensor(auth: UserTokenAuth, data: Json<CreateSensor>, conn: db::Conn) -> String {
    let sensor = SensorInsert {
        owner_id: auth.0.id,
        hardware_id: data.hardware_id,
    };
    match Sensor::insert(&sensor, conn.handler()) {
        Ok(_) => format!("success!"),
        Err(err) => format!("err: {}", err.to_string()),
    }
}

#[get("/private")]
fn private(auth: TokenAuth) -> String {
    format!(
        "Got private data for user with id {} using token \"{}\"",
        auth.0.user_id.expect("user had no id"),
        auth.0.token
    )
}

fn rocket() -> Rocket {
    rocket::ignite()
        .manage(db::init_pool())
        .mount(
            "/",
            routes![
                index,
                users,
                user,
                register,
                register_post,
                login,
                login_post,
                sensor,
            ],
        )
        .mount(
            "/api",
            routes![
                add_user,
                get_users,
                get_token,
                private,
                get_sensor_token,
                add_sensor,
                add_reading,
            ],
        )
        .mount("/static", routes![files])
        .attach(Template::fairing())
}

fn main() {
    rocket().launch();
}
