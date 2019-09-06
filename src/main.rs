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
    http::Status,
    response::{Flash, NamedFile, Redirect, Responder, Response},
    Request, Rocket,
};
use rocket_contrib::{
    json::JsonValue,
    templates::Template,
};
use std::path::{Path, PathBuf};

#[get("/<path..>")]
fn files(path: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new("static/").join(path)).ok()
}

#[get("/flash")]
fn set_flash() -> Flash<Redirect> {
    Flash::success(Redirect::to("/"), "this is a flash message!")
}

enum AnyResponder {
    Json(JsonValue),
    Flash(Flash<Redirect>),
}

impl<'r> Responder<'r> for AnyResponder {
    fn respond_to(self, req: &Request) -> ::std::result::Result<Response<'r>, Status> {
        match self {
            AnyResponder::Json(j) => j.respond_to(req),
            AnyResponder::Flash(f) => f.respond_to(req),
        }
    }
}

#[catch(401)]
fn not_authorized(req: &Request) -> AnyResponder {
    let msg = format!("not authorized to access {}", req.uri().path());
    if &req.uri().path()[0..4] == "/api" {
        AnyResponder::Json(json!({
            "status": "failure",
            "message": msg,
        }))
    } else {
        AnyResponder::Flash(Flash::error(Redirect::to("/login"), msg))
    }
}

fn rocket() -> Rocket {
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
                web::forgot_password,
                web::forgot_password_post,
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

fn main() {
    db::run_migrations();
    rocket().launch();
}
