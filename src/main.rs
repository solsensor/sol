#![feature(plugin)]
#![feature(custom_attribute)]
#![feature(custom_derive)]
#![allow(proc_macro_derive_resolution_fallback)]
#![feature(proc_macro_hygiene, decl_macro)]

extern crate base64;
extern crate rand;
#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate error_chain;

mod models;
mod schema;

use models::{
    Reading, ReadingInsert, ReadingQuery, Sensor, SensorInsert, SensorQuery, Token, TokenQuery,
    TokenType, User, UserInsert, UserQuery,
};
use rocket::{
    get,
    http::Status,
    post,
    request::{Form, FromRequest},
    response::{NamedFile, Redirect},
    routes, Outcome, Request, Rocket,
};
use rocket_contrib::{
    json::{Json, JsonValue},
    templates::Template,
};

use std::{
    io::{Error, ErrorKind},
    path::{Path, PathBuf},
    str::from_utf8,
};

mod echain {
    use rocket::{
        http::{ContentType, Status},
        request::Request,
        response::{Responder, Response},
    };
    use std::io::Cursor;

    error_chain!{}

    impl<'r> Responder<'r> for Error {
        fn respond_to(self, _: &Request) -> ::std::result::Result<Response<'r>, Status> {
            // Render the whole error chain to a single string
            let mut rslt = String::new();
            rslt += &format!("Error: {}", self);
            self.iter()
                .skip(1)
                .map(|ce| rslt += &format!(", caused by: {}", ce))
                .for_each(drop);

            // Create JSON response
            let resp = json!({
                "status": "failure",
                "message": rslt,
            })
            .to_string();

            // Respond. The `Ok` here is a bit of a misnomer. It means we
            // successfully created an error response
            Ok(Response::build()
                .status(Status::BadRequest)
                .header(ContentType::JSON)
                .sized_body(Cursor::new(resp))
                .finalize())
        }
    }
}

use echain::ResultExt;

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
fn users(conn: SolDbConn) -> Template {
    let users = User::all(&conn).ok();
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
fn user(email: String, conn: SolDbConn) -> Template {
    let user = User::by_email(&email, &conn).unwrap();
    let sensors = Sensor::find_for_user(user.id, &conn).ok();
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
fn sensor(id: i32, conn: SolDbConn) -> Template {
    let sensor = Sensor::find(id, &conn).unwrap();
    let readings = Reading::find_for_sensor(sensor.id, &conn).ok();
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
fn login_post(creds: Form<EmailPassword>, conn: SolDbConn) -> Result<Redirect, String> {
    let creds = creds.into_inner();
    let res = User::verify_password(&creds.email, &creds.password, &conn);
    match res {
        Ok(_user) => Ok(Redirect::to(uri!(user: creds.email))), // TODO set cookie or something
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
fn register_post(form: Form<UserInsert>, conn: SolDbConn) -> Result<Redirect, String> {
    let user = form.into_inner();
    let res = User::insert(&user, &conn);
    match res {
        Ok(_count) => Ok(Redirect::to(uri!(user: user.email))),
        Err(err) => Err(err.to_string()),
    }
}

#[get("/<path..>")]
fn files(path: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new("static/").join(path)).ok()
}

mod msg {
    use rocket::{
        http::{ContentType, Status},
        request::Request,
        response::{Responder, Response},
    };
    use std::io::Cursor;

    pub struct Message(String);

    impl Message {
        pub fn new(text: &str) -> Message {
            Message(text.to_string())
        }
    }

    impl<'r> Responder<'r> for Message {
        fn respond_to(self, _: &Request) -> ::std::result::Result<Response<'r>, Status> {
            // Create JSON response
            let resp = json!({
                    "status": "success",
                    "message": self.0,
                })
            .to_string();

            // Respond. The `Ok` here is a bit of a misnomer. It means we
            // successfully created an error response
            Ok(Response::build()
                .header(ContentType::JSON)
                .sized_body(Cursor::new(resp))
                .finalize())
        }
    }
}

use msg::Message;

#[post("/users/new", format = "application/json", data = "<user>")]
fn add_user(user: Json<UserInsert>, conn: SolDbConn) -> Result<Message, echain::Error> {
    let res = User::insert(&user.0, &conn);
    match res {
        Ok(_count) => Ok(Message::new("successfully inserted user")),
        Err(err) => Err(err.chain_err(|| "failed to insert user")),
    }
}

#[get("/users/all")]
fn get_users(conn: SolDbConn) -> String {
    format!(
        "all users: {:?}",
        User::all(&conn).expect("failed to get all users")
    )
}

#[post("/sensor_token", format = "application/json", data = "<sensor>")]
fn get_sensor_token(auth: UserTokenAuth, conn: SolDbConn, sensor: Json<SensorQuery>) -> String {
    let user = auth.0;
    let sensor = sensor.0;
    if user.id == sensor.owner_id {
        let token = Token::new_sensor_token(sensor);
        let res = Token::insert(&token, &conn);
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

#[post("/add_reading", format = "application/json", data = "<reading>")]
fn add_reading(auth: SensorTokenAuth, conn: SolDbConn, reading: Json<CreateReading>) -> String {
    let reading = ReadingInsert {
        id: None,
        voltage: reading.0.voltage,
        sensor_id: auth.0.id,
    };
    match Reading::insert(&reading, &conn) {
        Ok(_) => format!("success!"),
        Err(err) => format!("err: {}", err.to_string()),
    }
}

#[post("/token")]
fn get_token(auth: PasswordAuth, conn: SolDbConn) -> String {
    let user = auth.0;
    let token = Token::new_user_token(user);
    let res = Token::insert(&token, &conn);
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

impl<'a, 'r> FromRequest<'a, 'r> for PasswordAuth {
    type Error = String;
    fn from_request(req: &'a Request<'r>) -> Outcome<Self, (Status, String), ()> {
        let keys: Vec<_> = req.headers().get("Authorization").collect();
        if keys.len() != 1 {
            return Outcome::Failure((Status::Unauthorized, String::from("Missing Header")));
        }

        let words: Vec<String> = keys[0]
            .to_string()
            .split_whitespace()
            .map(String::from)
            .collect();
        if words.len() != 2 || words[0] != "Basic" {
            return Outcome::Failure((Status::Unauthorized, String::from("Malformed Header")));
        }

        let bytes = base64::decode(&words[1]).expect("failed to base64-decode");
        let words: Vec<String> = from_utf8(&bytes)
            .expect("failed to turn bytes to str")
            .to_string()
            .split(":")
            .map(|s| s.to_string())
            .collect();
        if words.len() != 2 {
            return Outcome::Failure((
                Status::Unauthorized,
                String::from("Malformed Email/Password"),
            ));
        }

        let conn: SolDbConn = req.guard().expect("req guard failed");
        let res = User::verify_password(&words[0], &words[1], &conn);
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

        let conn: SolDbConn = req.guard().expect("req guard failed");
        let tok = Token::find(&words[1], &conn).expect("could not find token");

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
                let conn: SolDbConn = req.guard().expect("request guard failed");
                let sensor_id = token.sensor_id.expect("token had no sensor id");
                match Sensor::find(sensor_id, &conn) {
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
                let conn: SolDbConn = req.guard().expect("request guard failed");
                let user_id = token.user_id.expect("token had no user id");
                match User::by_id(user_id, &conn) {
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
fn add_sensor(auth: UserTokenAuth, data: Json<CreateSensor>, conn: SolDbConn) -> String {
    let sensor = SensorInsert {
        owner_id: auth.0.id,
        hardware_id: data.hardware_id,
    };
    match Sensor::insert(&sensor, &conn) {
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

#[database("sqlite_sol")]
struct SolDbConn(diesel::SqliteConnection);

fn rocket() -> Rocket {
    rocket::ignite()
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
        .attach(SolDbConn::fairing())
}

fn main() {
    rocket().launch();
}
