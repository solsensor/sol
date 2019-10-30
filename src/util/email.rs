use crate::result::{Error, Result};
use rocket::{
    request::{FromRequest, Outcome},
    Request,
};
use rusoto_core::{region::Region, RusotoFuture};
use rusoto_ses::{Body, Content, Destination, Message, SendEmailRequest, Ses, SesClient};

pub struct Emailer {
    client: SesClient,
}

impl Emailer {
    pub fn send<S: Into<String>>(&self, to: S, subject: S, html: S) -> Result<()> {
        RusotoFuture::sync(self.client.send_email(SendEmailRequest {
            configuration_set_name: None,
            destination: Destination {
                bcc_addresses: None,
                cc_addresses: None,
                to_addresses: Some(vec![to.into()]),
            },
            message: Message {
                subject: Content {
                    charset: None,
                    data: subject.into(),
                },
                body: Body {
                    text: None,
                    html: Some(Content {
                        charset: None,
                        data: html.into(),
                        //data: ,
                    }),
                },
            },
            reply_to_addresses: None,
            return_path: None,
            return_path_arn: None,
            source: "devteam@solsensor.com".to_string(),
            source_arn: None,
            tags: None,
        }))?;
        Ok(())
    }
}

impl<'a, 'r> FromRequest<'a, 'r> for Emailer {
    type Error = Error;
    fn from_request(_req: &'a Request<'r>) -> Outcome<Self, Self::Error> {
        Outcome::Success(Emailer {
            client: SesClient::new(Region::UsEast1),
        })
    }
}
