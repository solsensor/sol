use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use diesel::sqlite::SqliteConnection;

use rocket::http::Status;
use rocket::request::{self, FromRequest};
use rocket::{Outcome, Request, State};

type ConnManager = ConnectionManager<SqliteConnection>;
type ConnPool = Pool<ConnManager>;
static DATABASE_URL: &'static str = "sol.sqlite";

pub fn init_pool() -> ConnPool {
    let manager = ConnManager::new(DATABASE_URL);
    Pool::new(manager).expect("db pool")
}

pub struct Conn(pub PooledConnection<ConnManager>);

impl Conn {
    pub fn handler(&self) -> &SqliteConnection {
        &self.0
    }
}

impl<'a, 'r> FromRequest<'a, 'r> for Conn {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
        let pool = request.guard::<State<ConnPool>>()?;
        match pool.get() {
            Ok(conn) => Outcome::Success(Conn(conn)),
            Err(_) => Outcome::Failure((Status::ServiceUnavailable, ())),
        }
    }
}
