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
