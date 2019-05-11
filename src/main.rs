#![feature(plugin)]
#![feature(custom_attribute)]
#![allow(proc_macro_derive_resolution_fallback)]
#![feature(proc_macro_hygiene, decl_macro)]

extern crate argon2rs;
extern crate base64;
extern crate chrono;
extern crate rand;
extern crate rocket_slog;
extern crate slog;
extern crate sloggers;
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
    fairing::Fairing,
    get,
    response::{Flash, NamedFile, Redirect},
    routes, Request, Rocket,
};
use rocket_contrib::{templates::Template};
use rocket_slog::{SlogFairing};
use sloggers::{
    terminal::{Destination, TerminalLoggerBuilder},
    types::Severity,
    Build,
};
use std::{
    path::{Path, PathBuf},
};

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

fn rocket(slog_fairing: impl Fairing) -> Rocket {
    rocket::ignite()
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
        .attach(slog_fairing)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    db::run_migrations();

    let mut builder = TerminalLoggerBuilder::new();
    builder.level(Severity::Debug);
    builder.destination(Destination::Stderr);
    let logger = builder.build()?;
    let slog_fairing = SlogFairing::new(logger);

    rocket(slog_fairing).launch();

    Ok(())
}
