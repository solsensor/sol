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

mod models;
mod result;
mod schema;
#[cfg(test)]
mod tests;

use crate::{
    models::{
        Reading, ReadingInsert, ReadingQuery, Sensor, SensorInsert, SensorQuery, Token, TokenQuery,
        TokenType, User, UserQuery,
    },
    result::{Error, Result},
};
use chrono::NaiveDateTime;
use diesel::Connection;
use diesel_migrations::embed_migrations;
use rocket::{
    get,
    http::{Cookie, Cookies, RawStr, Status},
    post,
    request::{FlashMessage, Form, FromFormValue, FromRequest},
    response::{Flash, NamedFile, Redirect},
    routes, Outcome, Request, Rocket,
};
use rocket_contrib::{json::Json, templates::Template};
use std::{
    path::{Path, PathBuf},
    str::from_utf8,
};

#[derive(Serialize)]
struct TemplateCtx {
    title: Option<String>,
    current_user: Option<UserQuery>,
    flash: Option<String>,
    users: Option<Vec<UserQuery>>,
    user: Option<UserQuery>,
    sensors: Option<Vec<SensorQuery>>,
    sensor: Option<SensorQuery>,
    readings: Option<Vec<ReadingQuery>>,
}

impl<'a, 'r> FromRequest<'a, 'r> for TemplateCtx {
    type Error = String;
    fn from_request(req: &'a Request<'r>) -> Outcome<Self, (Status, String), ()> {
        let current_user = req
            .guard()
            .success_or("failed")
            .ok()
            .map(|uc: UserCookieAuth| uc.0);
        let flash = req
            .guard()
            .success_or("failed")
            .ok()
            .map(|f: FlashMessage| format!("{}: {}", f.name(), f.msg()));
        let ctx = TemplateCtx {
            current_user,
            title: None,
            flash,
            user: None,
            users: None,
            sensors: None,
            sensor: None,
            readings: None,
        };
        Outcome::Success(ctx)
    }
}

#[get("/")]
fn index(mut ctx: TemplateCtx) -> Template {
    ctx.title = Some("Home".into());
    Template::render("index", ctx)
}

#[get("/users")]
fn users(mut ctx: TemplateCtx, conn: SolDbConn, _user: UserCookieAuth) -> Template {
    let users = User::all(&conn).ok();
    ctx.title = Some(String::from("Users"));
    ctx.users = users;
    Template::render("users", &ctx)
}

#[get("/user/<email>")]
fn user(mut ctx: TemplateCtx, email: String, conn: SolDbConn) -> Result<Template> {
    let user = User::by_email(&email, &conn)?;
    let sensors = Sensor::find_for_user(user.id, &conn).ok();

    ctx.title = Some(email);
    ctx.user = Some(user);
    ctx.sensors = sensors;
    Ok(Template::render("user", &ctx))
}

#[get("/sensor/<id>")]
fn sensor(mut ctx: TemplateCtx, id: i32, conn: SolDbConn) -> Result<Template> {
    let sensor = Sensor::find(id, &conn)?;
    let readings = Reading::find_for_sensor(sensor.id, &conn)?;
    let readings = Some(readings.into_iter().take(20).collect());
    ctx.title = Some(format!("sensor {}", id));
    ctx.sensor = Some(sensor);
    ctx.readings = readings;
    Ok(Template::render("sensor", &ctx))
}

#[get("/sensor/<id>/edit")]
fn sensor_edit(
    mut ctx: TemplateCtx,
    id: i32,
    conn: SolDbConn,
    _auth: UserCookieAuth,
) -> Result<Template> {
    let sensor = Sensor::find(id, &conn)?;
    ctx.title = Some(format!("sensor {} | edit", id));
    ctx.sensor = Some(sensor);
    Ok(Template::render("sensor_edit", &ctx))
}

#[derive(Deserialize, FromForm)]
struct SensorEdit {
    name: String,
    description: String,
}

#[post("/sensor/<id>/edit", data = "<form>")]
fn sensor_edit_post(
    auth: UserCookieAuth,
    form: Form<SensorEdit>,
    id: i32,
    conn: SolDbConn,
) -> Result<Flash<Redirect>> {
    let sensor = Sensor::find(id, &conn)?;
    if sensor.owner_id != auth.0.id {
        return Ok(Flash::error(
            Redirect::to(uri!(sensor: id)),
            "user does not own this sensor",
        ));
    }

    let form = form.0;
    Sensor::update(id, form.name, form.description, &conn)?;

    return Ok(Flash::success(
        Redirect::to(uri!(sensor: id)),
        "successfully updated sensor",
    ));
}

#[get("/login")]
fn login(mut ctx: TemplateCtx) -> Template {
    ctx.title = Some(String::from("Login"));
    Template::render("login", &ctx)
}

#[get("/change_password")]
fn change_password(mut ctx: TemplateCtx, _user: UserCookieAuth) -> Template {
    ctx.title = Some(String::from("Change Password"));
    Template::render("change_password", &ctx)
}

#[derive(Deserialize, FromForm)]
struct Password {
    password: String,
}

#[post("/change_password", data = "<form>")]
fn change_password_post(
    form: Form<Password>,
    conn: SolDbConn,
    user: UserCookieAuth,
    mut cookies: Cookies,
) -> Result<Redirect> {
    let user = user.0;
    let pwd = form.into_inner().password;
    User::update_password(user.id, pwd, &conn)?;
    cookies.remove_private(Cookie::named("user_token"));
    Ok(Redirect::to("/login"))
}

#[get("/logout")]
fn logout(mut cookies: Cookies) -> Redirect {
    cookies.remove_private(Cookie::named("user_token"));
    Redirect::to("/")
}

#[post("/login", data = "<creds>")]
fn login_post(
    creds: Form<EmailPassword>,
    conn: SolDbConn,
    mut cookies: Cookies,
) -> Result<Redirect> {
    let creds = creds.into_inner();
    let user = User::verify_password(&creds.email, &creds.password, &conn)?;
    let token = Token::new_user_token(&user);
    Token::insert(&token, &conn)?;
    cookies.add_private(Cookie::build("user_token", token.token).path("/").finish());
    Ok(Redirect::to(uri!(user: user.email)))
}

#[get("/register")]
fn register(mut ctx: TemplateCtx) -> Template {
    ctx.title = Some(String::from("Register"));
    Template::render("register", &ctx)
}

#[derive(Deserialize, FromForm)]
struct Register {
    email: String,
    password: String,
}

#[post("/register", data = "<form>")]
fn register_post(form: Form<Register>, conn: SolDbConn) -> Result<Redirect> {
    let form = form.into_inner();
    User::insert(form.email.clone(), form.password.clone(), &conn)
        .map(|_count| Redirect::to(uri!(user: form.email)))
}

#[get("/<path..>")]
fn files(path: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new("static/").join(path)).ok()
}

mod res {
    use rocket::{
        http::{ContentType, Status},
        request::Request,
        response::{Responder, Response},
    };
    use rocket_contrib::json::JsonValue;
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

    pub struct Data {
        message: String,
        data: JsonValue,
    }

    impl Data {
        pub fn new(msg: &str, data: JsonValue) -> Data {
            Data {
                message: msg.to_string(),
                data,
            }
        }
    }

    impl<'r> Responder<'r> for Data {
        fn respond_to(self, _: &Request) -> ::std::result::Result<Response<'r>, Status> {
            // Create JSON response
            let resp = json!({
                "status": "success",
                "message": self.message,
                "data": self.data,
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

use res::{Data, Message};

#[post("/users/new", format = "application/json", data = "<user>")]
fn add_user(user: Json<Register>, conn: SolDbConn) -> Result<Message> {
    User::insert(user.0.email, user.0.password, &conn)
        .map(|_| Message::new("successfully created user"))
}

#[get("/users/all")]
fn get_users(conn: SolDbConn) -> Result<Data> {
    User::all(&conn).map(|users| Data::new("found all users", json!({ "users": users })))
}

struct UnixEpochTime(NaiveDateTime);

impl<'v> FromFormValue<'v> for UnixEpochTime {
    type Error = &'v RawStr;
    fn from_form_value(form_value: &'v RawStr) -> std::result::Result<Self, Self::Error> {
        let unix = i64::from_form_value(form_value)?;
        Ok(UnixEpochTime(NaiveDateTime::from_timestamp(unix, 0)))
    }
}

impl<'de> serde::Deserialize<'de> for UnixEpochTime {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let unix = i64::deserialize(deserializer)?;
        let datetime = NaiveDateTime::from_timestamp(unix, 0);
        Ok(UnixEpochTime(datetime))
    }
}

#[get("/sensor/<id>/readings?<start>&<end>")]
fn get_readings(
    id: i32,
    start: UnixEpochTime,
    end: UnixEpochTime,
    conn: SolDbConn,
) -> Result<Data> {
    Reading::find_for_sensor_in_time_range(id, start.0, end.0, &conn).map(|readings| {
        Data::new(
            "found all readings for sensor in range",
            json!({ "readings": readings }),
        )
    })
}

#[derive(Deserialize)]
struct SensorHardwareId {
    hardware_id: i64,
}

#[post("/sensor_token", format = "application/json", data = "<sensor_hw_id>")]
fn get_sensor_token(
    auth: UserTokenAuth,
    conn: SolDbConn,
    sensor_hw_id: Json<SensorHardwareId>,
) -> Result<Data> {
    let user = auth.0;
    let hardware_id = sensor_hw_id.0.hardware_id;
    let sensor = Sensor::find_by_hardware_id(hardware_id, &conn)?;
    if user.id == sensor.owner_id {
        let token = Token::new_sensor_token(sensor);
        Token::insert(&token, &conn)
            .map(|_count| Data::new("got sensor token", json!({"token": token.token})))
    } else {
        Err(Error::NotSensorOwner)
    }
}

#[allow(non_snake_case)]
#[derive(Deserialize)]
struct CreateReading {
    timestamp: UnixEpochTime,
    peak_power_mW: f32,
    peak_current_mA: f32,
    peak_voltage_V: f32,
    temp_celsius: f32,
    batt_V: f32,
}

#[post("/add_reading", format = "application/json", data = "<reading>")]
fn add_reading(
    auth: SensorTokenAuth,
    conn: SolDbConn,
    reading: Json<CreateReading>,
) -> Result<Message> {
    let reading = ReadingInsert {
        sensor_id: auth.0.id,
        timestamp: reading.0.timestamp.0,
        peak_power_mW: reading.0.peak_power_mW,
        peak_current_mA: reading.0.peak_current_mA,
        peak_voltage_V: reading.0.peak_voltage_V,
        temp_celsius: reading.0.temp_celsius,
        batt_V: reading.0.batt_V,
    };
    Reading::insert(&reading, &conn).map(|_| Message::new("successfully added reading"))
}

#[post("/add_readings", format = "application/json", data = "<readings>")]
fn add_readings(
    auth: SensorTokenAuth,
    conn: SolDbConn,
    readings: Json<Vec<CreateReading>>,
) -> Result<Message> {
    let readings = readings
        .0
        .iter()
        .map(|r| ReadingInsert {
            sensor_id: auth.0.id,
            timestamp: r.timestamp.0,
            peak_power_mW: r.peak_power_mW,
            peak_current_mA: r.peak_current_mA,
            peak_voltage_V: r.peak_voltage_V,
            temp_celsius: r.temp_celsius,
            batt_V: r.batt_V,
        })
        .collect();
    Reading::insert_many(&readings, &conn).map(|_| Message::new("successfully added readings"))
}

#[post("/token")]
fn get_token(auth: BasicAuth, conn: SolDbConn) -> Result<Data> {
    let user = auth.0;
    let token = Token::new_user_token(&user);
    Token::insert(&token, &conn)
        .map(|_count| Data::new("got user token", json!({"token": token.token})))
}

struct BasicAuth(UserQuery);

#[derive(Deserialize, FromForm)]
struct EmailPassword {
    email: String,
    password: String,
}

impl<'a, 'r> FromRequest<'a, 'r> for BasicAuth {
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
            Ok(user) => Outcome::Success(BasicAuth(user)),
            Err(err) => Outcome::Failure((Status::Unauthorized, err.to_string())),
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

struct UserCookieAuth(UserQuery);
impl<'a, 'r> FromRequest<'a, 'r> for UserCookieAuth {
    type Error = String;
    fn from_request(req: &'a Request<'r>) -> Outcome<Self, (Status, String), ()> {
        let conn: SolDbConn = req.guard().expect("db req guard failed");
        let res = req
            .cookies()
            .get_private("user_token")
            .ok_or(Error::NoTokenInRequest)
            .map(|ck| ck.value().to_string())
            .and_then(|tok| User::by_token(&tok, &conn));
        match res {
            Ok(user) => Outcome::Success(UserCookieAuth(user)),
            Err(err) => Outcome::Failure((Status::Unauthorized, err.to_string())),
        }
    }
}

#[derive(Serialize, Deserialize)]
struct CreateSensor {
    hardware_id: i64,
}

#[post("/add_sensor", format = "application/json", data = "<data>")]
fn add_sensor(auth: UserTokenAuth, data: Json<CreateSensor>, conn: SolDbConn) -> Result<Message> {
    let sensor = SensorInsert {
        owner_id: auth.0.id,
        hardware_id: data.hardware_id,
    };
    Sensor::insert(&sensor, &conn).map(|_| Message::new("successfully added sensor"))
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
                change_password,
                change_password_post,
                logout,
                sensor,
                sensor_edit,
                sensor_edit_post,
                set_flash,
            ],
        )
        .mount(
            "/api",
            routes![
                add_user,
                get_users,
                get_token,
                get_sensor_token,
                add_sensor,
                add_reading,
                add_readings,
                get_readings,
            ],
        )
        .mount("/static", routes![files])
        .register(catchers![not_authorized])
        .attach(Template::fairing())
        .attach(SolDbConn::fairing())
}

embed_migrations!("./migrations");

fn main() {
    let conn = diesel::SqliteConnection::establish("./sol.sqlite").expect("error connecting to db");
    embedded_migrations::run(&conn).expect("failed to run migrations");
    rocket().launch();
}
