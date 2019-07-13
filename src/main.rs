#![feature(plugin)]
#![feature(custom_attribute)]
#![allow(proc_macro_derive_resolution_fallback)]
#![feature(proc_macro_hygiene, decl_macro)]

extern crate argon2rs;
extern crate base64;
extern crate chrono;
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
extern crate diesel_migrations;
extern crate insta;

mod api;
mod auth;
mod db;
mod models;
mod result;
mod schema;
#[cfg(test)]
mod tests;
mod util;
mod web;

use crate::db::SolDbConn;
use rocket::{
    response::{Flash, NamedFile, Redirect},
    Request, Rocket,
    config::{Config, Environment, Value},
};
use rocket_contrib::templates::Template;
use std::{collections::HashMap, path::{Path, PathBuf} };

#[get("/<path..>")]
fn files(path: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new("static/").join(path)).ok()
}

#[get("/flash")]
fn set_flash() -> Flash<Redirect> {
    Flash::success(Redirect::to("/"), "this is a flash message!")
}

#[catch(401)]
fn not_authorized(req: &Request) -> Flash<Redirect> {
    Flash::error(
        Redirect::to("/login"),
        format!("not authorized to access {}", req.uri().path()),
    )
}

fn rocket(cfg: Config) -> Rocket {
    rocket::custom(cfg)
        .mount(
            "/",
            routes![
                web::index,
                web::users,
                web::user,
                web::user_edit,
                web::user_edit_post,
                web::register,
                web::register_post,
                web::login,
                web::login_post,
                web::login_onetime,
                web::change_password,
                web::change_password_post,
                web::logout,
                web::sensor,
                web::sensor_edit,
                web::sensor_edit_post,
                set_flash,
            ],
        )
        .mount(
            "/api",
            routes![
                api::add_user,
                api::get_users,
                api::get_token,
                api::get_sensor_token,
                api::add_sensor,
                api::add_reading,
                api::add_readings,
                api::get_readings,
            ],
        )
        .mount("/static", routes![files])
        .register(catchers![not_authorized])
        .attach(Template::fairing())
        .attach(SolDbConn::fairing())
}

fn rocket_config(db_path: &str) -> Config {
    let mut database_cfg = HashMap::new();
    let mut databases = HashMap::new();
    database_cfg.insert("url", Value::from(db_path));
    databases.insert("sqlite_sol", Value::from(database_cfg));

    Config::build(Environment::Staging)
        .extra("databases", databases)
        .finalize()
        .expect("failed to build config")
}

fn main() {
    let db_path = "sol.sqlite";
    db::run_migrations(db_path);
    rocket(rocket_config(db_path)).launch();
}
