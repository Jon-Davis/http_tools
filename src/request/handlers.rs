use http::{Response, request::Request};
use crate::request::FilterStatus;
use http::response::Builder;
use bytes::Bytes;
use std::error::Error;

pub type HandlerResult = Result<Response<Bytes>, Box<dyn Error>>;

pub trait Handlers<R> {
    fn handle(request : &Request<R>, status : FilterStatus) -> HandlerResult;
}

pub struct DefaultHandlers();
impl<R> Handlers<R> for DefaultHandlers {
    fn handle(request : &Request<R>, status : FilterStatus) -> HandlerResult {

        match status {
            FilterStatus::FailFilterPath => Ok(Builder::new()
                    .status(404)
                    .version(request.version())
                    .header("Content-Length", 0)
                    .body(Bytes::new())?),
            FilterStatus::FailFilterMethod => Ok(Builder::new()
                    .status(405)
                    .version(request.version())
                    .header("Content-Length", 0)
                    .body(Bytes::new())?),
            _ => Ok(Builder::new()
                    .status(400)
                    .version(request.version())
                    .header("Content-Length", 0)
                    .body(Bytes::new())?),
        }
    }
}