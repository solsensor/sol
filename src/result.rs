use diesel::result::Error as DieselError;
use std::{convert::From, fmt};

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
