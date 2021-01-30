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
use http::request::Request;
use crate::request::Filter;
use crate::encoding::PercentEncodedStr;
/// The Extension trait provides additional methods to the Http Request type
pub trait RequestExtension<'a, R> {
    /// Creates an Option<&Request> that can be filtered
    /// on using the Filter trait. Whenever this filter struct is passed 
    /// through a filter function it will return Some if the inner 
    /// Request passed the filter, or None if the inner Request failed the filter. 
    fn filter_http(&'a self) -> Filter<'a, R>;
    fn get_path_var(&self, index : usize) -> Option<&str>;
}

impl<'a, R> RequestExtension<'a, R> for Request<R> {
    // Simply wrap a reference to the request in an Option
    fn filter_http(&'a self) -> Filter<'a, R> {
        Filter::new(self)
    }
    fn get_path_var(&self, index : usize) -> Option<&str> {
        self.uri().path().split('/').nth(index+1)
    }
}

/// Returns an iterator over a query string
/// 
/// # Example
/// ```
/// use http::request::Builder;
/// use http_tools::request::query_iter;
///
/// // given an  http request
/// let request = Builder::new()
///                 .uri("https://www.rust-lang.org/?one=two&three=four")
///                 .body(()).unwrap();
/// 
/// // use the http_tools function to create an iterator
/// for (key, value) in query_iter(&request){
///     println!("{} {}", key.inner(), value.inner())
/// }
/// 
/// // will print out 
/// // one two
/// // three four
/// ```
pub fn query_iter<R>(request : &Request<R>) -> impl Iterator<Item=(PercentEncodedStr<'_>, PercentEncodedStr<'_>)> {
    request.uri().query()
        .unwrap_or("")
        .split('&')
        .map(|q| {
            let mut q = q.split('=').fuse();
            (q.next(), q.next())
        })
        .filter(|(key, value)| key.is_some() && value.is_some())
        .map(|(key, value)| (PercentEncodedStr::new(key.unwrap()), PercentEncodedStr::new(value.unwrap())))
}
