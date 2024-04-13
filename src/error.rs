use anyhow;
use log::error;
use rocket::{
    http::Status,
    response::{self, Responder},
    Request,
};
use std::any::type_name_of_val;

#[derive(Debug)]
pub struct Error(pub anyhow::Error);

impl<E> From<E> for Error
where
    E: Into<anyhow::Error>,
{
    fn from(error: E) -> Self {
        Error(error.into())
    }
}

impl<'r, 'o: 'r> Responder<'r, 'o> for Error {
    fn respond_to(self, req: &'r Request<'_>) -> response::Result<'o> {
        let mut msg = format!("Handling error: {}", self.0);
        if let Some(source) = self.0.source() {
            msg = format!("{}; Caused by: {}", msg, source);
        }
        if self.0.downcast_ref::<reqwest::Error>().is_some() {
            error!("Reqwest error occurred: {}", msg);
            return Status::Unauthorized.respond_to(req);
        } else if self.0.downcast_ref::<diesel::result::Error>().is_some() {
            error!("Diesel error occurred: {}", msg);
            return Status::ServiceUnavailable.respond_to(req);
        } else if self.0.downcast_ref::<url::ParseError>().is_some() {
            error!("URL parse error occurred: {}", msg);
            return Status::InternalServerError.respond_to(req);
        } else {
            if let Some(e) = self.0.downcast_ref::<reqwest::Error>() {
                msg = format!("{}; type_name={}", msg, type_name_of_val(&e));
            }
            error!("Unhandled error: {}", msg);
            Status::InternalServerError.respond_to(req)
        }
    }
}
