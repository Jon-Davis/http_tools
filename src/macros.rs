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

/// Help reduce boilerplate of filtering on multiple handlers
/// 
/// The handle_fn macro can be used to simplify the code
/// when testing multiple handlers. The first argument is the
/// identifier of a handler function, and the second argument is an
/// expression that represents a Filter. If the filters succeed then 
/// the handler function will be called and passed the argument of the filter
/// and the macro will return the result of the handler function.
/// # Example
/// ```rust
/// #[macro_use] extern crate http_tools;
/// use http_tools::request::{Extension, Filter};
/// use http::request::{Request};
/// use http::response::{Builder, Response};
/// 
/// # fn post_handler(_req : &Request<()>) -> Response<()> {
/// #   return Builder::new().status(200).body(()).unwrap();
/// # } 
/// # fn get_handler(_req : &Request<()>) -> Response<()> {
/// #   return Builder::new().status(200).body(()).unwrap();
/// # } 
/// fn handle(req : &Request<()>) -> Response<()> {
///     handle_fn!(post_handler, req.filter()
///         .filter_path("/")
///         .filter_scheme("https")
///         .filter_method("POST"));
/// 
///     handle_fn!(get_handler, req.filter()
///         .filter_path("/")
///         .filter_scheme("https")
///         .filter_method("GET"));
/// 
///     Builder::new().status(405).body(()).unwrap()
/// }
/// ```
/// The macro expands into the following:
/// ```ignore
/// // handle_fn($handler, $filter)
/// match $filter {
///     Some(item) => return $handler(item),
///     _ => (),
/// }
/// ```
#[macro_export] macro_rules! handle_fn {
    ($handler:ident, $filter:expr) => {
        match $filter {
            Some(item) => return $handler(item),
            _ => (),
        }
    };
}

#[test]
fn test_handler() {

    let test_fn = |x| x % 2 == 0; 

    let test = |x| {
        handle_fn!(test_fn, x);
        false
    };

    assert!(test(Some(0)));
    assert!(!test(Some(1)));
    assert!(!test(None));
}
/*
#[test]
fn test_filter() {
    use crate::request::{Extension, Filter};
    use http::request::{Request};
    use http::response::{Builder, Response};
    let post_request = http::request::Builder::new().uri("https://www.rust-lang.org/").method("POST").body(()).unwrap();
    let get_request = http::request::Builder::new().uri("https://www.rust-lang.org/").method("GET").body(()).unwrap();
    let bad_request = http::request::Builder::new().uri("http://www.rust-lang.org/").method("GET").body(()).unwrap();
    
    fn test_handler(_req : &Request<()>) -> Response<()> {
        return Builder::new().status(200).body(()).unwrap();
    } 

    fn test(req : &Request<()>) -> Response<()> {
        handle_fn!(test_handler, req.filter()
            .filter_scheme("https")
            .filter_method("POST"));

        handle_fn!(test_handler, req.filter()
            .filter_scheme("https")
            .filter_method("GET"));

        Builder::new().status(400).body(()).unwrap()
    }
    assert!(test(&post_request).status() == 200u16);
    assert!(test(&get_request).status() == 200u16);
    assert!(test(&bad_request).status() == 400u16);
}
*/