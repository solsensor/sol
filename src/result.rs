use crate::{api::result::Error as ApiError, web::result::Error as WebError};
use diesel::result::Error as DieselError;
use rocket::{
    http::Status,
    request::Request,
    response::{Responder, Response},
};
use rusoto_core::RusotoError;
use rusoto_ses::SendEmailError;
use std::{convert::From, fmt};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Diesel(DieselError),
    DuplicateHardwareId(i64),
    IncorrectPassword,
    InvalidToken,
    MalformedToken,
    MissingToken,
    MissingBasicAuthHeader,
    MalformedBasicAuthHeader,
    WrongTokenType,
    NotSensorOwner,
    NoTokenInRequest,
    NotAdmin,
    DbConnectionFailed,
    SendEmail(SendEmailError),
    UnknownError(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let msg = match self {
            Error::Diesel(de) => format!("DieselError: {}", de),
            Error::DuplicateHardwareId(id) => {
                format!("active sensor with hardware id {} already exists", id)
            }
            Error::IncorrectPassword => "incorrect password".into(),
            Error::InvalidToken => "invalid token".into(),
            Error::MissingToken => "missing token".into(),
            Error::MissingBasicAuthHeader => "missing basic auth header".into(),
            Error::MalformedBasicAuthHeader => "malformed basic auth header".into(),
            Error::MalformedToken => "malformed token".into(),
            Error::WrongTokenType => "wrong token type".into(),
            Error::NotSensorOwner => "user is not the owner of this sensor".into(),
            Error::NoTokenInRequest => "failed to get auth token from request".into(),
            Error::NotAdmin => "user is not an admin".into(),
            Error::DbConnectionFailed => "failed to connect to the database".into(),
            Error::SendEmail(e) => format!("failed to send email: {}", e),
            Error::UnknownError(e) => format!("unknown error: {}", e),
        };
        write!(f, "{}", msg)
    }
}

impl From<DieselError> for Error {
    fn from(e: DieselError) -> Self {
        Error::Diesel(e)
    }
}

impl From<RusotoError<SendEmailError>> for Error {
    fn from(e: RusotoError<SendEmailError>) -> Self {
        match e {
            RusotoError::Service(se) => Error::SendEmail(se),
            ue => Error::UnknownError(format!("{}", ue)),
        }
    }
}

impl<'r> Responder<'r> for Error {
    fn respond_to(self, req: &Request) -> ::std::result::Result<Response<'r>, Status> {
        if &req.uri().path()[0..4] == "/api" {
            ApiError::from(self).respond_to(req)
        } else {
            WebError::from(self).respond_to(req)
        }
    }
}
