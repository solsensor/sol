use crate::result::Error as BaseError;
use rocket::{
    http::{ContentType, Status},
    request::Request,
    response::{Responder, Response},
};
use std::{
    convert::{From, Into},
    fmt,
    io::Cursor,
};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Error(BaseError);

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ApiError({})", self.0)
    }
}

impl<E: Into<BaseError>> From<E> for Error {
    fn from(err: E) -> Self {
        Error(err.into())
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
