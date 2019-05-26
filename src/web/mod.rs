use crate::{
    auth,
    db::SolDbConn,
    models::{Reading, ReadingQuery, Sensor, SensorQuery, Token, User, UserQuery},
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

mod result;
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
        };
        Outcome::Success(ctx)
    }
}

#[get("/")]
pub fn index(mut ctx: TemplateCtx) -> Template {
    ctx.title = Some("Home".into());
    Template::render("index", ctx)
}

#[get("/users")]
pub fn users(mut ctx: TemplateCtx, conn: SolDbConn, _user: auth::UserCookie) -> Template {
    let users = User::all(&conn).ok();
    ctx.title = Some(String::from("Users"));
    ctx.users = users;
    Template::render("users", &ctx)
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
    _auth: auth::UserCookie,
) -> WebResult<Template> {
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
    auth: auth::UserCookie,
    form: Form<UserEdit>,
    email: String,
    conn: SolDbConn,
) -> WebResult<Flash<Redirect>> {
    let user = User::by_email(&email, &conn)?;
    if user.id != auth.user().id {
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
    _auth: auth::UserCookie,
) -> WebResult<Template> {
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
    auth: auth::UserCookie,
    form: Form<SensorEdit>,
    id: i32,
    conn: SolDbConn,
) -> WebResult<Flash<Redirect>> {
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

#[get("/login")]
pub fn login(mut ctx: TemplateCtx) -> Template {
    ctx.title = Some(String::from("Login"));
    Template::render("login", &ctx)
}

#[get("/change_password")]
pub fn change_password(mut ctx: TemplateCtx, _user: auth::UserCookie) -> Template {
    ctx.title = Some(String::from("Change Password"));
    Template::render("change_password", &ctx)
}

#[derive(Deserialize, FromForm)]
pub struct Password {
    password: String,
}

#[post("/change_password", data = "<form>")]
pub fn change_password_post(
    form: Form<Password>,
    conn: SolDbConn,
    auth: auth::UserCookie,
    mut cookies: Cookies,
) -> WebResult<Redirect> {
    let user = auth.user();
    let pwd = form.into_inner().password;
    User::update_password(user.id, pwd, &conn)?;
    cookies.remove_private(Cookie::named("user_token"));
    Ok(Redirect::to("/login"))
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
pub fn login_onetime(token: String, conn: SolDbConn) -> WebResult<Redirect> {
    let user = User::by_onetime(&token, &conn)?;
    Ok(Redirect::to(uri!(user: user.email)))
}
