use diesel::result::Error as DieselError;
use rocket::{
    http::{ContentType, Status},
    request::Request,
    response::{Responder, Response},
};
use std::{convert::From, fmt, io::Cursor};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Diesel(DieselError),
    DuplicateHardwareId(i64),
    IncorrectPassword,
    WrongTokenType,
    NotSensorOwner,
    NoTokenInRequest,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let msg = match self {
            Error::Diesel(de) => format!("DieselError: {}", de),
            Error::DuplicateHardwareId(id) => {
                format!("active sensor with hardware id {} already exists", id)
            }
            Error::IncorrectPassword => "incorrect password".into(),
            Error::WrongTokenType => "wrong token type".into(),
            Error::NotSensorOwner => "user is not the owner of this sensor".into(),
            Error::NoTokenInRequest => "failed to get auth token from request".into(),
        };
        write!(f, "{}", msg)
    }
}

impl From<DieselError> for Error {
    fn from(e: DieselError) -> Self {
        Error::Diesel(e)
    }
}

impl<'r> Responder<'r> for Error {
    fn respond_to(self, _: &Request) -> ::std::result::Result<Response<'r>, Status> {
        // Create JSON response
        let resp = json!({
            "status": "failure",
            "message": self.to_string(),
        })
        .to_string();

        // Respond. The `Ok` here is a bit of a misnomer. It means we
        // successfully created an error response
        Ok(Response::build()
            .status(Status::BadRequest)
            .header(ContentType::JSON)
            .sized_body(Cursor::new(resp))
            .finalize())
    }
}
