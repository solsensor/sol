use crate::{
    auth,
    db::SolDbConn,
    models::{onetime_login, Reading, ReadingQuery, Sensor, SensorQuery, Token, User, UserQuery},
    result::Result,
    util::email::Emailer,
};
use rocket::{
    get,
    http::{Cookie, Cookies, Status},
    post,
    request::{FlashMessage, Form, FromRequest},
    response::{Flash, Redirect},
    Outcome, Request,
};
use rocket_contrib::templates::Template;

pub(crate) mod result;
use self::result::Result as WebResult;

#[derive(Serialize)]
pub struct TemplateCtx {
    title: Option<String>,
    current_user: Option<UserQuery>,
    flash: Option<String>,
    users: Option<Vec<UserQuery>>,
    user: Option<UserQuery>,
    sensors: Option<Vec<SensorQuery>>,
    sensor: Option<SensorQuery>,
    readings: Option<Vec<ReadingQuery>>,
    reading_count: Option<i64>,
    sensor_count: Option<i64>,
}

impl<'a, 'r> FromRequest<'a, 'r> for TemplateCtx {
    type Error = String;
    fn from_request(req: &'a Request<'r>) -> Outcome<Self, (Status, String), ()> {
        let current_user = req
            .guard()
            .success_or("failed")
            .ok()
            .map(|uc: auth::UserCookie| uc.user());
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
            reading_count: None,
            sensor_count: None,
        };
        Outcome::Success(ctx)
    }
}

#[get("/")]
pub fn index(user: Result<auth::UserCookie>) -> Redirect {
    match user {
        Ok(_) => Redirect::to(uri!(dashboard)),
        Err(_) => Redirect::to(uri!(landing)),
    }
}

#[get("/dashboard")]
pub fn dashboard(mut ctx: TemplateCtx, conn: SolDbConn) -> WebResult<Template> {
    let sensor_count = Sensor::count(&conn)?;
    let reading_count = Reading::count(&conn)?;

    ctx.title = Some("Dashboard".into());
    ctx.reading_count = Some(reading_count);
    ctx.sensor_count = Some(sensor_count);
    Ok(Template::render("dashboard", ctx))
}

#[get("/landing")]
pub fn landing(mut ctx: TemplateCtx, conn: SolDbConn) -> WebResult<Template> {
    let sensor_count = Sensor::count(&conn)?;
    let reading_count = Reading::count(&conn)?;

    ctx.title = Some("Home".into());
    ctx.reading_count = Some(reading_count);
    ctx.sensor_count = Some(sensor_count);
    Ok(Template::render("landing", ctx))
}

#[get("/users")]
pub fn users(
    mut ctx: TemplateCtx,
    conn: SolDbConn,
    auth: Result<auth::UserCookie>,
) -> WebResult<Template> {
    auth?;
    let users = User::all(&conn).ok();
    ctx.title = Some(String::from("Users"));
    ctx.users = users;
    Ok(Template::render("users", &ctx))
}

#[get("/user/<email>")]
pub fn user(mut ctx: TemplateCtx, email: String, conn: SolDbConn) -> WebResult<Template> {
    let user = User::by_email(&email, &conn)?;
    let sensors = Sensor::find_for_user(user.id, &conn).ok();

    ctx.title = Some(email);
    ctx.user = Some(user);
    ctx.sensors = sensors;
    Ok(Template::render("user", &ctx))
}

#[get("/user/<email>/edit")]
pub fn user_edit(
    mut ctx: TemplateCtx,
    email: String,
    conn: SolDbConn,
    auth: Result<auth::UserCookie>,
) -> WebResult<Template> {
    auth?;
    let user = User::by_email(&email, &conn)?;
    ctx.title = Some(format!("{} | edit", email));
    ctx.user = Some(user);
    Ok(Template::render("user_edit", &ctx))
}

#[derive(Deserialize, FromForm)]
pub struct UserEdit {
    email: String,
}

#[post("/user/<email>/edit", data = "<form>")]
pub fn user_edit_post(
    auth: Result<auth::UserCookie>,
    form: Form<UserEdit>,
    email: String,
    conn: SolDbConn,
) -> WebResult<Flash<Redirect>> {
    let auth = auth?;
    let user = User::by_email(&email, &conn)?;
    let editor = auth.user();
    if user.id != editor.id && !editor.superuser {
        return Ok(Flash::error(
            Redirect::to(uri!(user: email)),
            "not logged in as this user",
        ));
    }

    let form = form.0;
    User::update(user.id, &form.email, &conn)?;

    Ok(Flash::success(
        Redirect::to(uri!(user: &form.email)),
        "successfully updated user",
    ))
}

#[get("/sensor/<id>")]
pub fn sensor(mut ctx: TemplateCtx, id: i32, conn: SolDbConn) -> WebResult<Template> {
    let sensor = Sensor::find(id, &conn)?;
    let readings = Reading::find_for_sensor(sensor.id, &conn)?;
    let readings = Some(readings.into_iter().take(20).collect());
    ctx.title = Some(format!("sensor {}", id));
    ctx.sensor = Some(sensor);
    ctx.readings = readings;
    Ok(Template::render("sensor", &ctx))
}

#[get("/sensor/<id>/edit")]
pub fn sensor_edit(
    mut ctx: TemplateCtx,
    id: i32,
    conn: SolDbConn,
    auth: Result<auth::UserCookie>,
) -> WebResult<Template> {
    auth?;
    let sensor = Sensor::find(id, &conn)?;
    ctx.title = Some(format!("sensor {} | edit", id));
    ctx.sensor = Some(sensor);
    Ok(Template::render("sensor_edit", &ctx))
}

#[derive(Deserialize, FromForm)]
pub struct SensorEdit {
    name: String,
    description: String,
}

#[post("/sensor/<id>/edit", data = "<form>")]
pub fn sensor_edit_post(
    auth: Result<auth::UserCookie>,
    form: Form<SensorEdit>,
    id: i32,
    conn: SolDbConn,
) -> WebResult<Flash<Redirect>> {
    let auth = auth?;
    let sensor = Sensor::find(id, &conn)?;
    if sensor.owner_id != auth.user().id {
        return Ok(Flash::error(
            Redirect::to(uri!(sensor: id)),
            "user does not own this sensor",
        ));
    }

    let form = form.0;
    Sensor::update(id, form.name, form.description, &conn)?;

    Ok(Flash::success(
        Redirect::to(uri!(sensor: id)),
        "successfully updated sensor",
    ))
}

#[get("/sensor/<id>/deactivate")]
pub fn sensor_deactivate(
    mut ctx: TemplateCtx,
    id: i32,
    conn: SolDbConn,
    auth: Result<auth::UserCookie>,
) -> WebResult<Template> {
    auth?;
    let sensor = Sensor::find(id, &conn)?;
    ctx.title = Some(format!("sensor {} | deactivate", id));
    ctx.sensor = Some(sensor);
    Ok(Template::render("sensor_deactivate", &ctx))
}

#[post("/sensor/<id>/deactivate")]
pub fn sensor_deactivate_post(
    auth: Result<auth::UserCookie>,
    id: i32,
    conn: SolDbConn,
) -> WebResult<Flash<Redirect>> {
    let auth = auth?;
    let sensor = Sensor::find(id, &conn)?;
    if sensor.owner_id != auth.user().id {
        return Ok(Flash::error(
            Redirect::to(uri!(sensor: id)),
            "user does not own this sensor",
        ));
    }

    Sensor::deactivate(&conn, id)?;

    Ok(Flash::success(
        Redirect::to(uri!(sensor: id)),
        "successfully deactivated sensor",
    ))
}

#[get("/login")]
pub fn login(mut ctx: TemplateCtx) -> Template {
    ctx.title = Some(String::from("Login"));
    Template::render("login", &ctx)
}

#[get("/change_password")]
pub fn change_password(
    mut ctx: TemplateCtx,
    auth: Result<auth::UserCookie>,
) -> WebResult<Template> {
    auth?;
    ctx.title = Some(String::from("Change Password"));
    Ok(Template::render("change_password", &ctx))
}

#[derive(Deserialize, FromForm)]
pub struct Password {
    password: String,
}

#[post("/change_password", data = "<form>")]
pub fn change_password_post(
    form: Form<Password>,
    conn: SolDbConn,
    auth: Result<auth::UserCookie>,
    mut cookies: Cookies,
) -> WebResult<Redirect> {
    let user = auth?.user();
    let pwd = form.into_inner().password;
    User::update_password(user.id, pwd, &conn)?;
    cookies.remove_private(Cookie::named("user_token"));
    Ok(Redirect::to("/login"))
}

#[derive(Deserialize, FromForm)]
pub struct Email {
    email: String,
}

#[get("/forgot_password")]
pub fn forgot_password(mut ctx: TemplateCtx) -> Template {
    ctx.title = Some(String::from("Forgot Password"));
    Template::render("forgot_password", &ctx)
}

#[post("/forgot_password", data = "<form>")]
pub fn forgot_password_post(
    form: Form<Email>,
    emailer: Result<Emailer>,
    conn: SolDbConn,
) -> WebResult<Flash<Redirect>> {
    let email = &form.0.email;
    let user = User::by_email(email, &conn)?;
    let token = onetime_login::create(user.id, &conn)?;

    emailer?.send("ryan@ryanchipman.com", "Password Reset", &format!("<html><body>Reset password at this link: <a href=\"https://dev.solsensor.com/login/onetime/{}\">Reset Password</a></body></html>", token))?;

    Ok(Flash::success(
        Redirect::to("/forgot_password"),
        "temporary password has been emailed",
    ))
}

#[get("/logout")]
pub fn logout(mut cookies: Cookies) -> Redirect {
    cookies.remove_private(Cookie::named("user_token"));
    Redirect::to("/")
}

#[derive(Deserialize, FromForm)]
pub struct EmailPassword {
    email: String,
    password: String,
}

#[post("/login", data = "<creds>")]
pub fn login_post(
    creds: Form<EmailPassword>,
    conn: SolDbConn,
    mut cookies: Cookies,
) -> WebResult<Redirect> {
    let creds = creds.into_inner();
    let user = User::verify_password(&creds.email, &creds.password, &conn)?;
    let token = Token::new_user_token(&user);
    Token::insert(&token, &conn)?;
    cookies.add_private(Cookie::build("user_token", token.token).path("/").finish());
    Ok(Redirect::to(uri!(user: user.email)))
}

#[get("/register")]
pub fn register(mut ctx: TemplateCtx) -> Template {
    ctx.title = Some(String::from("Register"));
    Template::render("register", &ctx)
}

#[derive(Deserialize, FromForm)]
pub struct Register {
    email: String,
    password: String,
}

#[post("/register", data = "<form>")]
pub fn register_post(form: Form<Register>, conn: SolDbConn) -> WebResult<Redirect> {
    let form = form.into_inner();
    User::insert(form.email.clone(), form.password.clone(), &conn)?;
    Ok(Redirect::to(uri!(user: form.email)))
}

#[get("/login/onetime/<token>")]
pub fn login_onetime(token: String, conn: SolDbConn, mut cookies: Cookies) -> WebResult<Redirect> {
    let user = User::by_onetime(&token, &conn)?;
    onetime_login::delete(&token, &conn)?;
    let token = Token::new_user_token(&user);
    Token::insert(&token, &conn)?;
    cookies.add_private(Cookie::build("user_token", token.token).path("/").finish());
    Ok(Redirect::to(uri!(change_password)))
}
