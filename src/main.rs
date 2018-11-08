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

use models::{Sensor, SensorInsert, SensorQuery};
use models::{Token, TokenQuery, TokenType};
use models::{User, UserInsert, UserQuery};

use rocket::data::{self, Data, FromData};
use rocket::http::Status;
use rocket::request::{Form, FromRequest};
use rocket::response::{NamedFile, Redirect};
use rocket::{Outcome, Request, Rocket};
use rocket_contrib::{Json, Template, Value};

use std::path::{Path, PathBuf};

#[derive(Serialize)]
struct TemplateCtx {
    title: String,
    users: Option<Vec<UserQuery>>,
    user: Option<UserQuery>,
}

#[get("/")]
fn index() -> Template {
    let ctx = TemplateCtx {
        title: String::from("Home"),
        users: None,
        user: None,
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
    };
    Template::render("users", &ctx)
}

#[get("/user/<email>")]
fn user(email: String, conn: db::Conn) -> Template {
    let user = User::by_email(&email, conn.handler()).ok();
    let ctx = TemplateCtx {
        title: email,
        users: None,
        user,
    };
    Template::render("user", &ctx)
}

#[get("/login")]
fn login() -> Template {
    let ctx = TemplateCtx {
        title: String::from("Login"),
        user: None,
        users: None,
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
    format!("all users: {:?}", User::all(conn.handler()).unwrap())
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
        let login: Json<EmailPassword> = Json::from_data(req, data).unwrap();
        let login = login.into_inner();

        let conn: db::Conn = req.guard().unwrap();
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

        let conn: db::Conn = req.guard().unwrap();
        let tok = Token::find(&words[1], conn.handler()).unwrap();

        Outcome::Success(TokenAuth(tok))
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
                let conn: db::Conn = req.guard().unwrap();
                let user_id = token.user_id.unwrap();
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
        auth.0.user_id.unwrap(),
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
            ],
        )
        .mount("/static", routes![files])
        .attach(Template::fairing())
}

fn main() {
    rocket().launch();
}
