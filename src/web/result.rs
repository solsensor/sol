use crate::result::Error as BaseError;
use rocket::{
    http::Status,
    request::Request,
    response::{Flash, Redirect, Responder, Response},
};
use std::{
    convert::{From, Into},
    fmt,
};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Error(BaseError);

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "WebError({})", self.0)
    }
}

impl<E: Into<BaseError>> From<E> for Error {
    fn from(err: E) -> Self {
        Error(err.into())
    }
}

impl<'r> Responder<'r> for Error {
    fn respond_to(self, req: &Request) -> ::std::result::Result<Response<'r>, Status> {
        let res: Flash<Redirect> = Flash::error(Redirect::to("/"), self.to_string());
        res.respond_to(req)
    }
}
