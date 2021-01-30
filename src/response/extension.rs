// MIT License
// 
// Copyright (c) 2019 Jonathon Davis
// 
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
// 
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software. 
// 
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.
use bytes::Bytes;
use http::{StatusCode, response::Response};
use  anyhow::Error;

/// The Extension trait provides additional methods to the Http Response type
pub trait ResponseExtension {
    /// Creates an Option<&Response> that can be filtered
    /// on using the Filter trait. Whenever this filter struct is passed 
    /// through a filter function it will return Some if the inner 
    /// Response passed the filter, or None if the inner Response failed the filter. 
    fn filter_http(&self) -> Option<&Self>;

    /// Converts an error caused by a handler function into a response. Handlers use
    /// the anyhow crate for returning errors, anyhow allows the attachment of contexts
    /// to errors. `from_error(err)` will pull the StatusCode and &str contexts and use those to create
    /// a http response
    /// # Example
    /// ```
    /// use http::{request, StatusCode, response::Response, Method};
    /// use bytes::Bytes;
    /// use http_tools::{request::RequestExtension, response::ResponseExtension};
    /// use anyhow::Context;
    /// # use futures::executor::block_on;
    /// # let request = request::Builder::new().uri("https://www.rust-lang.org/")
    /// #    .method(Method::GET)
    /// #    .body(Bytes::new())
    /// #    .unwrap();
    ///
    /// # block_on(async {
    /// let sv1 = request.filter_http()
    ///     .filter_path("/") 
    ///     .async_handle(|_| async move {
    ///         // An unrecoverable error occurs while handling a Request
    ///         u8::from_str_radix("abc", 10)
    ///             .context(StatusCode::IM_A_TEAPOT) // Set Status of Response
    ///             .context("Short and spout!")?; // Set body of Response
    ///         unreachable!();
    ///     });
    ///    
    /// let response = sv1.await
    ///         .unwrap_or_else(|_| Ok(Response::<Bytes>::from_status(StatusCode::NOT_FOUND)))
    ///         .unwrap_or_else(Response::<Bytes>::from_error); // magic happens here <---
    /// 
    /// assert!(response.status() == StatusCode::IM_A_TEAPOT);
    /// assert!(response.body() == "Short and spout!");
    /// # });
    /// ```
    fn from_error(err : Error) -> Response<Bytes> {
        let status = match err.downcast_ref::<StatusCode>(){
            Some(s) => s,
            _ => &StatusCode::INTERNAL_SERVER_ERROR,
        };
        let body = match err.downcast_ref::<&str>(){
            Some(s) => s,
            _ => status.canonical_reason().unwrap_or(""),
        };
        Response::builder().status(status).body(Bytes::from(body))
            .unwrap_or_else(|_| Self::from_status(*status))
    }

    fn from_status(status: StatusCode) -> Response<Bytes> {
        let mut response = Response::default();
        *response.status_mut() = status;
        response
    }
}

impl<R> ResponseExtension for Response<R> {
    // Simply wrap a refrence to the response in an Option
    fn filter_http(&self) -> Option<&Self> {
        Some(self)
    }
}
