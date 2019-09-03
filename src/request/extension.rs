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
///     println!("{} {}", key, value)
/// }
/// 
/// // will print out 
/// // one two
/// // three four
/// ```
pub fn query_iter<'a, R>(request : &'a Request<R>) -> impl 'a + Iterator<Item=(&'a str, &'a str)> {
    request.uri().query()
        .unwrap_or("")
        .split("&")
        .map(|q| {
            let mut q = q.split('=').fuse();
            (q.next(), q.next())
        })
        .filter(|(key, value)| key.is_some() && value.is_some())
        .map(|(key, value)| (key.unwrap(), value.unwrap())) 
}
