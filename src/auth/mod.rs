use crate::{
    db::SolDbConn,
    models::{Sensor, SensorQuery, Token, TokenQuery, TokenType, User, UserQuery},
    result::Error,
};
use rocket::{
    http::Status,
    request::{FromRequest, Outcome},
    Request,
};
use std::str::from_utf8;

pub struct UserCookie(UserQuery);

impl UserCookie {
    pub fn user(self) -> UserQuery {
        self.0
    }
}

impl<'a, 'r> FromRequest<'a, 'r> for UserCookie {
    type Error = Error;
    fn from_request(req: &'a Request<'r>) -> Outcome<Self, Self::Error> {
        let conn: SolDbConn = req.guard().expect("db req guard failed");
        let res = req
            .cookies()
            .get_private("user_token")
            .ok_or(Error::NoTokenInRequest)
            .map(|ck| ck.value().to_string())
            .and_then(|tok| User::by_token(&tok, &conn));
        match res {
            Ok(user) => Outcome::Success(UserCookie(user)),
            Err(err) => Outcome::Failure((Status::BadRequest, err.into())),
        }
    }
}

pub struct Basic(UserQuery);

impl Basic {
    pub fn user(self) -> UserQuery {
        self.0
    }
}

impl<'a, 'r> FromRequest<'a, 'r> for Basic {
    type Error = Error;
    fn from_request(req: &'a Request<'r>) -> Outcome<Self, Self::Error> {
        let keys: Vec<_> = req.headers().get("Authorization").collect();
        if keys.len() != 1 {
            return Outcome::Failure((Status::BadRequest, Error::MissingBasicAuthHeader));
        }

        let words: Vec<String> = keys[0]
            .to_string()
            .split_whitespace()
            .map(String::from)
            .collect();
        if words.len() != 2 || words[0] != "Basic" {
            return Outcome::Failure((Status::BadRequest, Error::MalformedBasicAuthHeader));
        }

        let bytes = base64::decode(&words[1]).expect("failed to base64-decode");
        let words: Vec<String> = from_utf8(&bytes)
            .expect("failed to turn bytes to str")
            .to_string()
            .split(":")
            .map(|s| s.to_string())
            .collect();
        if words.len() != 2 {
            return Outcome::Failure((Status::BadRequest, Error::MalformedBasicAuthHeader));
        }

        let conn: SolDbConn = req.guard().expect("req guard failed");
        let res = User::verify_password(&words[0], &words[1], &conn);
        match res {
            Ok(user) => Outcome::Success(Basic(user)),
            Err(err) => Outcome::Failure((Status::BadRequest, err)),
        }
    }
}

struct AuthToken(TokenQuery);

impl<'a, 'r> FromRequest<'a, 'r> for AuthToken {
    type Error = Error;
    fn from_request(req: &'a Request<'r>) -> Outcome<Self, Self::Error> {
        let keys: Vec<_> = req.headers().get("Authorization").collect();
        if keys.len() != 1 {
            return Outcome::Failure((Status::BadRequest, Error::MissingToken));
        }

        let words: Vec<String> = keys[0]
            .to_string()
            .split_whitespace()
            .map(String::from)
            .collect();
        if words.len() != 2 || words[0] != "bearer" {
            return Outcome::Failure((Status::BadRequest, Error::MalformedToken));
        }

        let conn: SolDbConn = req.guard().expect("req guard failed");
        let tok = Token::find(&words[1], &conn).expect("could not find token");

        Outcome::Success(AuthToken(tok))
    }
}

pub struct SensorToken(SensorQuery);

impl SensorToken {
    pub fn sensor(self) -> SensorQuery {
        self.0
    }
}

impl<'a, 'r> FromRequest<'a, 'r> for SensorToken {
    type Error = Error;
    fn from_request(req: &'a Request<'r>) -> Outcome<Self, Self::Error> {
        let token: AuthToken = req.guard()?;
        let token = token.0;

        match TokenType::from_string(token.type_) {
            TokenType::User => Outcome::Failure((Status::BadRequest, Error::WrongTokenType)),
            TokenType::Sensor => {
                let conn: SolDbConn = req.guard().expect("request guard failed");
                let sensor_id = token.sensor_id.expect("token had no sensor id");
                match Sensor::find(sensor_id, &conn) {
                    Ok(sensor) => Outcome::Success(SensorToken(sensor)),
                    Err(_err) => Outcome::Failure((Status::BadRequest, Error::InvalidToken)),
                }
            }
        }
    }
}

pub struct UserToken(UserQuery);

impl UserToken {
    pub fn user(self) -> UserQuery {
        self.0
    }
}

impl<'a, 'r> FromRequest<'a, 'r> for UserToken {
    type Error = Error;
    fn from_request(req: &'a Request<'r>) -> Outcome<Self, Self::Error> {
        let token: AuthToken = req.guard()?;
        let token = token.0;

        match TokenType::from_string(token.type_) {
            TokenType::Sensor => Outcome::Failure((Status::BadRequest, Error::WrongTokenType)),
            TokenType::User => {
                let conn: SolDbConn = req.guard().expect("request guard failed");
                let user_id = token.user_id.expect("token had no user id");
                match User::by_id(user_id, &conn) {
                    Ok(user) => Outcome::Success(UserToken(user)),
                    Err(_err) => Outcome::Failure((Status::BadRequest, Error::InvalidToken)),
                }
            }
        }
    }
}

pub struct AdminToken(UserQuery);

impl<'a, 'r> FromRequest<'a, 'r> for AdminToken {
    type Error = Error;

    fn from_request(req: &'a Request<'r>) -> Outcome<Self, Self::Error> {
        let token: UserToken = req.guard()?;
        let user = token.0;
        if user.superuser {
            Outcome::Success(AdminToken(user))
        } else {
            Outcome::Failure((Status::BadRequest, Error::NotAdmin))
        }
    }
}
