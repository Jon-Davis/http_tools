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
use std::error::Error;
use bytes::Bytes;
use http::{StatusCode, response::Response};

/// The Extension trait provides additional methods to the Http Response type
pub trait ResponseExtension {
    /// Creates an Option<&Response> that can be filtered
    /// on using the Filter trait. Whenever this filter struct is passed 
    /// through a filter function it will return Some if the inner 
    /// Response passed the filter, or None if the inner Response failed the filter. 
    fn filter_http(&self) -> Option<&Self>;

    fn from_error<E : Error>(err : &E, status: StatusCode) -> Response<Bytes> {
        Response::builder().status(status).body(Bytes::from(format!("{}", err)))
            .unwrap_or_else(|_| Self::from_status(status))
    }

    fn from_boxed_error(err : Box<dyn Error>, status: StatusCode) -> Response<Bytes> {
        Response::builder().status(status).body(Bytes::from(format!("{}", err)))
            .unwrap_or_else(|_| Self::from_status(status))
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
