#![feature(plugin)]
#![feature(custom_attribute)]
#![feature(custom_derive)]
#![plugin(rocket_codegen)]
#![allow(proc_macro_derive_resolution_fallback)]

extern crate rand;
extern crate rocket;
#[macro_use] extern crate rocket_contrib;
#[macro_use] extern crate diesel;
#[macro_use] extern crate serde_derive;

mod db;
mod models;
mod schema;

use models::{User, UserInsert, UserQuery};
use models::{Token, TokenInsert, TokenQuery};

use rand::Rng;

use rocket::{Rocket,Outcome,Request};
use rocket::http::Status;
use rocket::request::{Form,FromRequest};
use rocket::response::{NamedFile,Redirect};
use rocket_contrib::{Json, Template, Value};

use std::iter;
use std::path::{Path, PathBuf};

#[derive(Serialize)]
struct TemplateCtx {
    title: String,
    users: Option<Vec<UserQuery>>,
    user: Option<UserQuery>,
}

#[get("/")]
fn index() -> Template {
    let ctx = TemplateCtx{
        title: String::from("Home"),
        users: None,
        user:None,
    };
    Template::render("index", &ctx)
}

#[get("/users")]
fn users(conn: db::Conn) -> Template {
    let users = User::all(conn.handler()).ok();
    let ctx = TemplateCtx{
        title: String::from("Users"),
        user: None,
        users,
    };
    Template::render("users", &ctx)
}

#[get("/user/<email>")]
fn user(email: String, conn: db::Conn) -> Template {
    let user = User::by_email(&email, conn.handler()).ok();
    let ctx = TemplateCtx{
        title: email,
        users: None,
        user,
    };
    Template::render("user", &ctx)
}

#[get("/register")]
fn register() -> Template {
    let ctx = TemplateCtx {
        title: String::from("register"),
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
        Err(err) => Err(err.to_string())
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

#[derive(Deserialize)]
struct Email {
    email: String,
}

#[post("/login", format = "application/json", data = "<data>")]
fn get_token(data: Json<Email>, conn: db::Conn) -> String {
    let email = data.0.email;
    let user = User::by_email(&email, conn.handler()).unwrap();

    let mut rng = rand::thread_rng();
    let token_str: String = iter::repeat(())
        .map(|()| rng.sample(rand::distributions::Alphanumeric))
        .take(64)
        .collect();

    let token = TokenInsert{
        token: token_str.clone(),
        user_id: user.id
    };
    let res = Token::insert(&token, conn.handler());
    match res {
        Ok(_count) => token_str,
        Err(err) => format!("failed: {}", err.to_string())
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

        let words: Vec<String> = keys[0].to_string()
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

#[get("/private")]
fn private(auth: TokenAuth) -> String {
    format!("Got private data for user with id {} using token \"{}\"", auth.0.user_id, auth.0.token)
}

fn rocket() -> Rocket {
    rocket::ignite()
        .manage(db::init_pool())
        .mount("/", routes![index,users,user,register,register_post])
        .mount("/static", routes![files])
        .mount("/api", routes![add_user,get_users,get_token,private])
        .attach(Template::fairing())
}

fn main() {
    rocket().launch();
}
