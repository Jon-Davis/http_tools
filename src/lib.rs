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

/*! 
The http_tools crate aims to provide additional functions on the types found in the 
http crate. This crate aims to only provide functions that do not cause heap allocations and
to be as lightweight as possible. Additonally the crate is compatiable with any library or
web framework that also uses the http crate.

# Filters
The crate provides tools that expand upon the Request and Response types found in the http crate.
allowing for the quick creation of an http router.
```
# use http::request::Builder;
use http_tools::request::{Extension, Filter};
# let request = Builder::new()
#                .uri("https://www.rust-lang.org/item/rust?cool=rust&also+cool=go")
#                .extension(-1i32)
#                .method("POST")
#                .header("content-type", "application/x-www-form-urlencoded")
#                .header("content-length", "0")
#                .body(()).unwrap();

// standard Http::request::Request
request
    // Creates an Option<&Request>, each fiter returns Some(req) if it passes and None if it fails
    .filter()
    // match the path /item/{} where {} is a wild card
    .filter_path("/item/{}")
    // request has the method of type POST
    .filter_method("POST")
    // The header has the key content-type with a value of application/x-www-form-urlencoded
    .filter_header("content-type", "application/x-www-form-urlencoded")
    // The {} wild card can be used to filter headers aswell
    .filter_header("content-length", "{}")
    // The query has the key cool with the value rust
    .filter_query("cool", "rust")
    // the wild card {} can be used in queries, filters do not decode uri encodings
    .filter_query("also+cool", "{}")
    // custom filters can be applied, and will be given the request and return a bool
    .filter_custom(|req| req.extensions().get::<i32>().is_some())
    // The request has a scheme of https
    .filter_scheme("https")
    // filters simply return std Option where Some means pass and None means failed
    .and_then(|_request| Some("I passed the test!"));
```
# Routing Service
The crate provides some helpful macros to reduce the boiler plate of creating a routing service
```rust
#[macro_use] extern crate http_tools;
use http_tools::request::{Extension, Filter};
use http::request::{Request};
use http::response::{Builder, Response};

# fn post_handler(_req : &Request<()>) -> Response<()> {
#   return Builder::new().status(200).body(()).unwrap();
# } 
# fn get_handler(_req : &Request<()>) -> Response<()> {
#   return Builder::new().status(200).body(()).unwrap();
# } 
# fn service_handler(_req : &Request<()>) -> Response<()> {
#   return Builder::new().status(200).body(()).unwrap();
# }
// routes the request to the proper handler and returns the response
fn router(req : &Request<()>) -> Response<()> {
    // Handles Post requests to root uri
    handle_fn!(post_handler, req.filter()
        .filter_path("/")
        .filter_method("POST"));

    // Handles Get requests to root uri
    handle_fn!(get_handler, req.filter()
        .filter_path("/")
        .filter_method("GET"));

    // Handles Get requests to some_service/WILDCARD 
    handle_fn!(service_handler, req.filter()
        .filter_path("some_service/{}")
        .filter_method("GET"));

    // Returns method not found
    Builder::new().status(405).body(()).unwrap()
}
```
# Iterators
The crate provides some useful iterators
```
# use http::request::Builder;
use http_tools::request::query_iter;
# // given an  http request
# let request = Builder::new()
#                .uri("https://www.rust-lang.org/?one=two&three=four")
#                .body(()).unwrap();

// use the http_tools function to create an iterator given an http::request::Request
for (key, value) in query_iter(&request){
    println!("{} {}", key, value)
}
```
*/
extern crate http;

pub mod request;
pub mod response;
pub mod interface;
pub mod encoding;
mod macros;
