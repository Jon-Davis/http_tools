The http_tools crate aims to provide additional functions on the types found in the 
http crate. 

# Filters
The crate provides tools that expand upon the Request and Response types found in the http crate.
allowing for the quick creation of an async http router.
```rust
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
 ```rust
use http::{StatusCode, response::{Response, Builder}};
use http_tools::{response::ResponseExtension, request::RequestExtension};
use futures::future::TryFutureExt;
use bytes::Bytes;

// GET /item/{:string} -> Got any {:string}
let sv1 = request.filter_http()
        .filter_path("/item/{}")
        .filter_method("GET")
        .async_handle(|req| async move {
            let input = req.get_path_var(1).unwrap();
            let output = format!("Got any {}?", input);
            Ok(Builder::new().body(Bytes::from(output)).unwrap())
        });

// GET /hello/{:string} -> Hello {:string}
let sv2  = |_| {
request.filter_http()
    .filter_path("/hello/{}")
    .filter_method("GET")
    .async_handle(|req| async move {
        let input = req.get_path_var(1).unwrap();
        let output = format!("Hello {}!", input);
        Ok(Builder::new().body(Bytes::from(output)).unwrap())
    })
};

// Lazy evaluate paths, set default 404 and 500 errors
let response = sv1.or_else(sv2).await
    // If neither of the Filters passed, return a 404 NOT_FOUND response
    .unwrap_or_else(|_| Ok(Response::<Bytes>::from_status(StatusCode::NOT_FOUND)))
    // If the service returned an Error, create a Response from the Errors Display
    .unwrap_or_else(Response::<Bytes>::from_error);

// Got any grapes?
assert!(response.body() == "Got any grapes?");

```
# Other tools
The crate also provides some useful functions for working with the Http types
```rust
use http::request::Builder;
use http_tools::request::query_iter;
// given an  http request
let request = Builder::new()
            .uri("https://www.rust-lang.org/?one=two&three=fo+ur")
            .body(()).unwrap();

// use the http_tools function to create an iterator given an http::request::Request
for (key, val) in query_iter(&request){
    assert!(key == "one" || key == "three");
    assert!(val == "two" || val == "fo ur");
}
```