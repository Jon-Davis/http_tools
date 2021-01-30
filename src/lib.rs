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
http crate. 

# Filters
The crate provides tools that expand upon the Request and Response types found in the http crate.
allowing for the quick creation of an async http router.
```
# use http::request::Builder;
# use http::{request, response, method::Method};
use http_tools::request::{RequestExtension, Filter};
# let request = Builder::new()
#                .uri("https://www.rust-lang.org/item/baseball?cool=rust")
#                .method("POST")
#                .body(()).unwrap();
// http::Request
request.filter_http()
    .filter_path("/item/{}") 
    .filter_query("cool", "rust")
    .filter_method(Method::POST)
    .handle(|_| Ok(Default::default()));
```
# Async Router
Using an async handler, http tools can call async functions when the filter passes all of it's checks. 
Multiple filters can be used on a single Request, routing the function to the first successful filter.
 ```
# use http::request::Builder;
# use futures::executor::block_on; 
use http::{request, response};
use http_tools::request::{RequestExtension, Filter};
use bytes::Bytes;
# block_on(async {
# let request = Builder::new()
#                .uri("https://www.rust-lang.org/item/grapes")
#                .method("POST")
#                .body(()).unwrap();

let sv1 = request.filter_http()
            .filter_path("/item/{}") 
            .async_handle(|req| async move {
                let input = req.get_path_var(1).unwrap();
                let output = format!("Got any {}", input);
                Ok(response::Builder::new().body(Bytes::from(output)).unwrap())
            }).await;

let sv2 = request.filter_http()
            .filter_path("/hello/{}") 
            .async_handle(|req| async move {
                let input = req.get_path_var(1).unwrap();
                let output = format!("Hello {}", input);
                Ok(response::Builder::new().body(Bytes::from(output)).unwrap())
            }).await;

// handlers return Some if the filter produced an output
sv1.or(sv2);
# });
```
# Other tools
The crate also provides some useful functions for working with the Http types
```
# use http::request::Builder;
use http_tools::request::query_iter;
# // given an  http request
let request = Builder::new()
                .uri("https://www.rust-lang.org/?one=two&three=fo+ur")
                .body(()).unwrap();

// use the http_tools function to create an iterator given an http::request::Request
for (key, val) in query_iter(&request){
    assert!(key == "one" || key == "three");
    assert!(val == "two" || val == "fo ur");
}
```
*/
extern crate http;

pub mod request;
pub mod response;
pub mod encoding;
