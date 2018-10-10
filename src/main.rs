#![feature(plugin)]
#![feature(custom_attribute)]
#![plugin(rocket_codegen)]
#![allow(proc_macro_derive_resolution_fallback)]

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

use models::{User, UserInsert, UserQuery};

use rocket::response::NamedFile;
use rocket::Rocket;
use rocket_contrib::{Json, Template, Value};

use std::path::{Path, PathBuf};

#[derive(Serialize)]
struct TemplateCtx {
    title: String,
    users: Option<Vec<UserQuery>>,
}

#[get("/")]
fn index() -> Template {
    let ctx = TemplateCtx{
        title: String::from("Home"),
        users: None,
    };
    Template::render("index", &ctx)
}

#[get("/users")]
fn users(conn: db::Conn) -> Template {
    let users = User::all(conn.handler()).ok();
    let ctx = TemplateCtx{
        title: String::from("Users"),
        users,
    };
    Template::render("users", &ctx)
}

#[get("/<path..>")]
fn files(path: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new("static/").join(path)).ok()
}

#[post("/users/new", format = "application/json", data = "<user>")]
fn add_user(user: Json<UserInsert>, conn: db::Conn) -> Json<Value> {
    let res = User::insert(user.0, conn.handler());
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

fn rocket() -> Rocket {
    rocket::ignite()
        .manage(db::init_pool())
        .mount("/", routes![index,users])
        .mount("/static", routes![files])
        .mount("/api", routes![add_user,get_users])
        .attach(Template::fairing())
}

fn main() {
    rocket().launch();
}
