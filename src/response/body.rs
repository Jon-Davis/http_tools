use std::fmt::{Display};

use bytes::Bytes;


/// A simple wrapper around Bytes, used by the `ResponseExtention::from_error` Method.
///
/// Http_tool handler functions return an anyhow::Result, because of this context to errors
/// can be added to convert them into a http::Response. The `ResponseExtention::from_error`
/// method will take an error with a context of ResponseBody and use the value as the body
/// for the http::Response
/// # Example
/// ```
/// use bytes::Bytes;
/// use http::{StatusCode, Response};
/// use http_tools::response::{ResponseExtension, ResponseBody};
/// use anyhow::Context;
///
/// let err = u8::from_str_radix("abc", 10)
///             .context(StatusCode::IM_A_TEAPOT) // Set Status of Response
///             .context(ResponseBody::new("Short and spout!")) // Set body of Response
///             .unwrap_err(); 
///
/// let response = Response::<Bytes>::from_error(err);
/// 
/// assert!(response.status() == StatusCode::IM_A_TEAPOT);
/// assert!(response.body() == "Short and spout!");
/// ```
#[derive(Debug)]
pub struct ResponseBody(pub(crate) Bytes);

impl ResponseBody {
    pub fn new<G : Into<Bytes>>(src : G) -> Self {
        ResponseBody(src.into())
    }
}

impl Display for ResponseBody {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ResponseBody <...>")
    }
}

#[test]
fn test_response_body(){
use http::{StatusCode, Response};
use crate::response::ResponseExtension;
use anyhow::Context;

let err = u8::from_str_radix("abc", 10)
            .context(StatusCode::IM_A_TEAPOT) // Set Status of Response
            .context(ResponseBody::new("Short and spout!")) // Set body of Response
            .unwrap_err(); 

let response = Response::<Bytes>::from_error(err);

assert!(response.status() == StatusCode::IM_A_TEAPOT);
assert!(response.body() == "Short and spout!");
}