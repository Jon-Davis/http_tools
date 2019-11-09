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

/* ============================================================================================ */
/*     Document Structure                                                                       */
/*          Filter Trait                                                                        */
/*          impl Filter for Option<Request>                                                     */
/*          Test Cases                                                                          */
/* ============================================================================================ */
use http::request::Request;
use http::header::HeaderValue;
use http::method::Method;
use crate::request::query_iter;

/* ============================================================================================ */
/*     Filter Trait                                                                             */
/* ============================================================================================ */

/// The filter trait allows for the easy filtering of requests. They can be chained
/// together to create more complex filters. 
/// 
/// Filters were designed to be used in making an http router, but can be used in many more ways. 
/// The trait always outputs an Option<&Request>. If the option is Some then the underlying 
/// filter applies to the request, if the request is None then request did not pass the filter. 
/// The filter trait is implemented on both a Request and an Option<&Request>
/// # Syntax
/// ```
/// # use http::request::Builder;
/// use http_tools::request::{Filter, Extension};
/// # let request = Builder::new()
/// #                .uri("https://www.rust-lang.org/item/rust?cool=rust&also+cool=go")
/// #                .extension(-1i32)
/// #                .method("POST")
/// #                .header("content-type", "application/x-www-form-urlencoded")
/// #                .header("content-length", "0")
/// #                .body(()).unwrap();
/// 
/// // given an http::request::Request
/// request
///     // Creates an Option<&Request>, each fiter returns Some(req) if it passes and None if it fails
///     .filter()
///     // match the path /item/{} where {} is a wild card
///     .filter_path("/item/{}")
///     // request has the method of type POST
///     .filter_method("POST")
///     // The header has the key content-type with a value of application/x-www-form-urlencoded
///     .filter_header("content-type", "application/x-www-form-urlencoded")
///     // The {} wild card can be used to filter headers aswell
///     .filter_header("content-length", "{}")
///     // The query has the key cool with the value rust
///     .filter_query("cool", "rust")
///     // the wild card {} can be used in queries, filters do not decode uri encodings
///     .filter_query("also+cool", "{}")
///     // custom filters can be applied, and will be given the request and return a bool
///     .filter_custom(|req| req.extensions().get::<i32>().is_some())
///     // The request has a scheme of https
///     .filter_scheme("https")
///     // filters simply return std Option where Some means pass and None means failed
///     .and_then(|_request| Some("I passed the test!"));
///  ```
/// 
/// Since all filters simply take a refrence to a Request and return an Option, the filters can be applied
/// in a number of different ways. For example the below router ensures that the Request is authenticated before
/// routing it.
/// ```
/// # use http::request::{Request, Builder};
/// use http_tools::request::{Extension, Filter};
/// # let request = Builder::new()
/// #                .uri("https://www.rust-lang.org/item/rust?cool=rust&also+cool=go")
/// #                .body(()).unwrap();
/// # fn some_handler<R>(req : &Request<R>) -> Option<()> {None};
/// # fn some_other_handler<R>(req : &Request<R>) -> Option<()>{None};
/// 
/// // The following filter must pass for any of the other handlers to pass
/// let parent_filter = request.filter()
///     .filter_header("Authorization", "type token");
/// 
/// // check to see if the request is equal, if so call the handler function
/// parent_filter
///     .filter_path("/")
///     .and_then(|req| some_handler(req));
/// 
/// // if the above function didn't match we check the next one and if this one
/// // matches then we call some other handler function
/// parent_filter
///     .filter_path("/{}")
///     .and_then(|req| some_other_handler(req));
/// ```
/// 
/// If you only ever want 1 handler function to be applied, then it is more efficent to test
/// each filter afterwards, and return the output
/// ```
/// # use http::{Request, Response, StatusCode};
/// use http_tools::request::{Extension, Filter};
/// # let request = Request::builder()
/// #                .uri("https://www.rust-lang.org/item/rust?cool=rust&also+cool=go")
/// #                .body(()).unwrap();
/// # fn some_handler<R>(req : &Request<R>) -> Response<()> { Response::builder().body(()).unwrap() };
/// # fn some_other_handler<R>(req : &Request<R>) -> Response<()>{ Response::builder().body(()).unwrap() };
/// 
/// fn mux<R>(request : &Request<R>) -> Response<()> {
///     // Check the first filter
///     let filter = request.filter()
///         .filter_method("GET")
///         .filter_path("/");
///     if let Some(req) = filter {
///         return some_handler(req);
///     }
///     
///     // check the second filter
///     let filter = request.filter()
///         .filter_method("GET")
///         .filter_path("/{}");
///     if let Some(req) = filter {
///         return some_other_handler(req);
///     }
/// 
///     return Response::builder()
///         .status(StatusCode::NOT_FOUND)
///         .body(())
///         .unwrap()
/// }
///
/// ```
pub trait Filter<'a, R> {
    /// Checks to see if the request has the specified key and value. The wildcard '{}'
    /// pattern can be used in either the key or the value string. The function returns
    /// Some(request) if the header with the key and value are found or None if they are
    /// absent. 
    /// # Example
    /// ```
    /// use http::request::Builder;
    /// use http_tools::request::{Extension, Filter};
    /// 
    /// // Request Builder found in http crate
    /// let request = Builder::new()
    ///                 .uri("https://www.rust-lang.org/")
    ///                 .header("key", "value")
    ///                 .body(()).unwrap();
    /// 
    /// // matches when the key is key and value is value
    /// let filter = request.filter().filter_header("key", "value");
    /// assert!(filter.is_some());
    /// 
    /// // matches when the key exists
    /// let filter = request.filter().filter_header("key", "{}");
    /// assert!(filter.is_some());
    /// ```
    fn filter_header<T>(self, key : &str, value : T) -> Self where T : PartialEq<HeaderValue> + PartialEq<&'static str>;
    /// Checks to see if the requests path matches the specified pattern. The wildcard '{}'
    /// pattern can be used can be used to match any text between foward slashes
    /// so '/{}' will match '/any' but not '/any/more'
    /// # Example
    /// ```
    /// use http::request::Builder;
    /// use http_tools::request::{Extension, Filter};
    /// 
    /// // Request Builder found in http crate
    /// let request = Builder::new()
    ///                     .uri("https://www.rust-lang.org/var/static")
    ///                     .body(()).unwrap();
    /// 
    /// // this will match because the paths are exact
    /// let filter = request.filter().filter_path("/var/static");
    /// assert!(filter.is_some());
    /// 
    /// // this will match because the wildcard '{}' will match var
    /// let filter = request.filter().filter_path("/{}/static");
    /// assert!(filter.is_some());
    /// ```
    fn filter_path(self, pattern : &str) -> Self;
    /// Checks to see if the requests path begins with the specified pattern. The wildcard '{}'
    /// pattern can be used to match any text between foward slashes.
    /// # Example
    /// ```
    /// use http::request::Builder;
    /// use http_tools::request::{Extension, Filter};
    /// 
    /// // Request Builder found in http crate
    /// let request = Builder::new()
    ///                     .uri("https://www.rust-lang.org/var/static")
    ///                     .body(()).unwrap();
    /// 
    /// // this will match any path
    /// let filter = request.filter().filter_path_prefix("/");
    /// assert!(filter.is_some());
    /// 
    /// // this will match any path atleast 1 in size
    /// let filter = request.filter().filter_path_prefix("/{}");
    /// assert!(filter.is_some());
    /// 
    /// // this will match any path that starts with var
    /// let filter = request.filter().filter_path_prefix("/var");
    /// assert!(filter.is_some());
    /// 
    /// // Note: This will NOT pass as each subpath must be complete
    /// let filter = request.filter().filter_path_prefix("/v"); // '/v' doesn't match '/var'
    /// assert!(filter.is_none());
    /// ```
    fn filter_path_prefix(self, pattern : &str) -> Self;
    /// Checks to see if the request has the inputed method.
    /// # Example
    /// ```
    /// use http::request::Builder;
    /// use http_tools::request::{Extension, Filter};
    /// // Request Builder found in http crate
    /// let request = Builder::new()
    ///                     .uri("https://www.rust-lang.org/")
    ///                     .method("GET")
    ///                     .body(()).unwrap();
    /// 
    /// // this will match any path
    /// let filter = request.filter().filter_method("GET");
    /// assert!(filter.is_some());
    /// 
    /// // this will not 
    /// let filter = request.filter().filter_method("POST");
    /// assert!(filter.is_none());
    /// ```
    fn filter_method<T>(self, method : T) -> Self where T : PartialEq<Method>;
    /// Checks to see if the uri contains a query with the given key and value. The wildcard '{}'
    /// pattern can be used to match any key or value.
    /// # Example
    /// ```
    /// use http::request::Builder;
    /// use http_tools::request::{Extension, Filter};
    /// // Request Builder found in http crate
    /// let request = Builder::new()
    ///                     .uri("https://www.rust-lang.org/?cool=rust&also+cool=go")
    ///                     .body(()).unwrap();
    /// 
    /// // this will match as the value of cool is equal to rust
    /// let filter = request.filter().filter_query("cool", "rust");
    /// assert!(filter.is_some());
    /// 
    /// // this will match because the key of cool exists
    /// let filter = request.filter().filter_query("cool", "{}");
    /// assert!(filter.is_some());
    /// 
    /// // this will match because go is a value in the query
    /// let filter = request.filter().filter_query("{}", "go");
    /// assert!(filter.is_some());
    /// 
    /// // this will NOT match because filter_query does not decode it's arguments
    /// let filter = request.filter().filter_query("also cool", "go");
    /// assert!(filter.is_none());
    /// // this will work
    /// let filter = request.filter().filter_query("also+cool", "go");
    /// assert!(filter.is_some());
    /// ```
    fn filter_query(self, key : &str, value : &str) -> Self;
    /// Checks to see if the request has given scheme
    /// # Example
    /// ```
    /// use http::request::Builder;
    /// use http_tools::request::{Extension, Filter};
    /// // Request Builder found in http crate
    /// let request = Builder::new()
    ///                     .uri("https://www.rust-lang.org/")
    ///                     .body(()).unwrap();
    /// 
    /// // this will match
    /// let filter = request.filter().filter_scheme("https");
    /// assert!(filter.is_some());
    /// // this will not 
    /// let filter = request.filter().filter_scheme("http");
    /// assert!(filter.is_none());
    /// ```
    fn filter_scheme(self, scheme : &str) -> Self;
    /// filter_custom allows for a custom function filter. The filter will be given a &Request and
    /// will output a bool. if the bool is true, then function returns Some, if it is false then the
    /// function will return None
    /// # Example
    /// ```
    /// use http::request::Builder;
    /// use http_tools::request::{Extension, Filter};
    /// // Request Builder found in http crate
    /// let request = Builder::new()
    ///                     .extension(-1i32)
    ///                     .uri("https://www.rust-lang.org/")
    ///                     .body(()).unwrap();
    /// 
    /// // this will match as the request has an extension of type i32
    /// let filter = request.filter().filter_custom(|req| req.extensions().get::<i32>().is_some());
    /// assert!(filter.is_some());
    /// ```
    fn filter_custom(self, func : fn(&Request<R>) -> bool) -> Self;
}

/* ============================================================================================ */
/*     impl Filter for Option<Request>                                                          */
/* ============================================================================================ */

// The filter trait implentation for Option<&Request> does the actuall filtering
// It takes in an Option<&Request> and outputs an Option<&Request> in order to allow
// for the chaining of multiple filters. If a filter function returns Some that means
// that the request passed through the filter, if a filter function returns None that
// means the request did not pass the filter.
impl<'a, R> Filter<'a, R> for Option<&Request<R>>{
    // The filter_query function for Option<&Request> first checks to see that the value of
    // self is Some, then it calls request::query_iter to retrieve an iterator over the query
    // arguments. If the key is equal to the inputed key or equal to the wild card '{}'
    // and the value is equal to the inputed value or equal to the wild card '{}' then the
    // function will return Some and pass along it's refence, otherwise the 
    fn filter_query(self, key : &str, value : &str) -> Self {
        // since the filter functions can return none, we can't perform any work (and shouldn't)
        // if a previous filter invalidated the Request
        if let Some(request) = self {
            // iterate through the querys
            for (q_key,q_value) in query_iter(request){
                // if the key == q_key or {} and the value == q_value or {} then the pattern
                // matches and we can return the refrence
                if (key == "{}" || key == q_key) && (value == "{}" || value == q_value) {
                    return Some(request);
                }
            }
        }
        // If the filter broke out, or self was None then return None
        None
    }
    // The filter_header function for Option<&Request> first checks to see that the value
    // of self is Some. Then it checks the key, if the key is a wild card then the values
    // will need to be iterated through to check to see if they match, if the key is not
    // a wild card then we can call the get function on the Requests HeaderMap for the key.
    fn filter_header<T>(self, key : &str, value : T) -> Self where T : PartialEq<HeaderValue> + PartialEq<&'static str> {
        // since the filter functions can return none, we can't perform any work (and shouldn't)
        // if a previous filter invalidated the Request
        if let Some(request) = self {
            // Check to see if the key is the wildcard token of '{}'
            if key == "{}" {
                // retrieve the headers map
                let map = request.headers();
                // If the value is {} and there are entries in the header map
                // the return Some request as any value would match
                if value == "{}" && map.len() > 0 {
                    return Some(request);
                }
                // Iterate through the different values to see if any values
                // match the inputed value
                for v in map.values() {
                    // if the values match return Some
                    if value == *v {
                        return Some(request);
                    }
                }
            } else {
                // Get the key and check if it's value is equal to the inputed value
                // otherwise fall through to the end and return None
                match request.headers().get(key) {
                    Some(v) if value == *v || value == "{}" => return Some(request),
                    _ => (),
                }
            }
        }
        // If the filter broke out, or self was None then return None
        None
    }
    // The filter_path function for Option<&Request> first checks to see that the value of
    // self is Some, then it checks to see if the path of the request matches the pattern.
    // if they aren't an exact match then the filter checks to see if there are any variable
    // wildcards {}.
    fn filter_path(self, pattern : &str) -> Self {
        // since the filter functions can return none, we can't perform any work (and shouldn't)
        // if a previous filter invalidated the Request
        if let Some(request) = self {
            // get the path from the uri
            let path = request.uri().path();
            // check to see if the path and pattern equal eachother
            if path == pattern {
                return Some(request);
            } else {
                // create two iterators split on the forward slash for both
                // the pattern given as an argument and the actual path of 
                // the request being filtered
                let mut split_pattern = pattern.split('/');
                let mut split_path = path.split('/');
                loop {
                    // call next on each of the iterators
                    let pattern_item = split_pattern.next();
                    let path_item = split_path.next();
                    match (pattern_item, path_item) {
                        // if they both have a result check to see if the pattern is a wildcard or they equal eachother
                        (Some(pattern), Some(path)) if pattern != "{}" && pattern != path => return None,
                        // if the pattern or path end before one another then they are not the same length, thus not equal
                        (None, Some(_)) | (Some(_), None) => return None,
                        // if both the pattern and path end at the same time then they have been equal up to this point
                        // and are assumed to be equal
                        (None, None) => return Some(request),
                        // if both the pattern and path are equal or the pattern is {} continue with the loop
                        _ => ()
                    }
                }
            }
        }
        // If the filter broke out, or self was None then return None
        None
    }
    // The filter_path_prefix function for Option<&Request> first checks to see that the value of
    // self is Some, then it checks to see if the path of the request matches the pattern.
    // if they aren't an exact match then the filter checks to see if there are any variable
    // wildcards {}.
    fn filter_path_prefix(self, pattern : &str) -> Self {
        // since the filter functions can return none, we can't perform any work (and shouldn't)
        // if a previous filter invalidated the Request
        if let Some(request) = self {
            // get the path from the uri
            let path = request.uri().path();
            // check to see if the path and pattern equal eachother
            if path == pattern {
                return Some(request);
            } else {
                // create two iterators split on the forward slash for both
                // the pattern given as an argument and the actual path of 
                // the request being filtered
                let mut split_pattern = pattern.split('/');
                let mut split_path = path.split('/');
                loop {
                    // call next on each of the iterators
                    let pattern_item = split_pattern.next();
                    let path_item = split_path.next();
                    match (pattern_item, path_item) {
                        // since we are using the split operator if the pattern ends with a /
                        // then there will be a lingering "". check to make sure it is a lingering
                        // "" and not one in the middle of the pattern
                        (Some(pattern), Some(path)) if pattern == "" && pattern != path => {
                            match split_pattern.next() {
                                Some(_) => return None,
                                None => return Some(request),
                            }
                        }
                        // check to see if the pattern and path differ
                        (Some(pattern), Some(path)) if pattern != "{}" && pattern != path => return None,
                        // if the path ends before the pattern then it is not a prefix
                        (Some(_), None) => return None,
                        // if the pattern ends before the path or they both end at the same time then its a prefix
                        (None, Some(_)) | (None, None) => return Some(request),
                        // if they equal eachother continue through the loop
                        _ => ()
                    }
                }
            }
        }
        // If the filter broke out, or self was None then return None
        None
    }
    // The filter_method function for Option<&Request> first checks to see that the value of
    // self is Some, then checks to see if the request method is equal to the inputed method.
    fn filter_method<T>(self, method : T) -> Self where T : PartialEq<Method> {
        // since the filter functions can return none, we can't perform any work (and shouldn't)
        // if a previous filter invalidated the Request
        if let Some(request) = self {
            // check to see if the request method equals the method argument
            if method == *request.method() {
                return self;
            }
        }
        // If the filter broke out, or self was None then return None
        None
    }
    // The filter_scheme function for Option<&Request> first checks to see that the value of
    // self is Some, then checks to see if the request scheme is equal to the inputed scheme.
    fn filter_scheme(self, scheme : &str) -> Self {
        if let Some(request) = self {
            // check to see if the request scheme equals the scheme argument
            match request.uri().scheme_str() {
                Some(s) if s == scheme => return Some(request),
                _ => (),
            }
        }
        // If the filter broke out, or self was None then return None
        None
    }
    // The filter_scheme function for Option<&Request> first checks to see that the value of
    // self is Some, then checks to see if the request scheme is equal to the inputed scheme.
    fn filter_custom(self, func : fn(&Request<R>) -> bool) -> Self {
        if let Some(request) = self {
            let result = func(request);
            if result {
                return self;
            }
        }
       None
    }
}

/* ============================================================================================ */
/*     Test Cases                                                                               */
/* ============================================================================================ */
#[test]
fn test_root_route() {
    use http::request::Builder;
    use crate::request::Extension;
    let request = Builder::new().uri("https://www.rust-lang.org/").body(()).unwrap();
    let filter = request.filter().filter_path("/");
    assert!(filter.is_some());
}

#[test]
fn test_full_route() {
    use http::request::Builder;
    use crate::request::Extension;
    let request = Builder::new().uri("https://www.rust-lang.org/this/is/a/longer/path").body(()).unwrap();
    let filter = request.filter().filter_path("/this/is/a/longer/path");
    assert!(filter.is_some());
}

#[test]
fn test_var_route() {
    use http::request::Builder;
    use crate::request::Extension;
    let request = Builder::new().uri("https://www.rust-lang.org/var/static").body(()).unwrap();
    let filter = request.filter().filter_path("/{}/static");
    assert!(filter.is_some());
}

#[test]
fn test_partial_route() {
    use http::request::Builder;
    use crate::request::Extension;
    let request = Builder::new().uri("https://www.rust-lang.org/this/is/different").body(()).unwrap();
    let filter = request.filter().filter_path("/this/is");
    assert!(filter.is_none());
}

#[test]
fn test_pattern_route() {
    use http::request::Builder;
    use crate::request::Extension;
    let request = Builder::new().uri("https://www.rust-lang.org/").body(()).unwrap();
    let filter = request.filter().filter_path("this/is/longer");
    assert!(filter.is_none());
}

#[test]
fn test_path_route() {
    use http::request::Builder;
    use crate::request::Extension;
    let request = Builder::new().uri("https://www.rust-lang.org/this/is/longer").body(()).unwrap();
    let filter = request.filter().filter_path("/");
    assert!(filter.is_none());
}

#[test]
fn test_path_prefix() {
    use http::request::Builder;
    use crate::request::Extension;
    let request = Builder::new().uri("https://www.rust-lang.org/this/is/longer").body(()).unwrap();
    let filter = request.filter().filter_path_prefix("/");
    assert!(filter.is_some());
    let filter = request.filter().filter_path_prefix("/this/is");
    assert!(filter.is_some());
    let filter = request.filter().filter_path_prefix("/{}");
    assert!(filter.is_some());
    let filter = request.filter().filter_path_prefix("/this/is/longer/than/the/original");
    assert!(filter.is_none());
    let filter = request.filter().filter_path_prefix("/th");
    assert!(filter.is_none());
}

#[test]
fn test_different_route() {
    use http::request::Builder;
    use crate::request::Extension;
    let request = Builder::new().uri("https://www.rust-lang.org/this/is/different").body(()).unwrap();
    let filter = request.filter().filter_path("/not/even/close");
    assert!(filter.is_none());
}

#[test]
fn test_header() {
    use http::request::Builder;
    use crate::request::Extension;
    let request = Builder::new().uri("https://www.rust-lang.org/").header("key", "value").body(()).unwrap();
    let filter = request.filter().filter_header("key", "value");
    assert!(filter.is_some());
    let filter = request.filter().filter_header("key", "{}");
    assert!(filter.is_some());
    let filter = request.filter().filter_header("{}", "value");
    assert!(filter.is_some());
    let filter = request.filter().filter_header("{}", "{}");
    assert!(filter.is_some());
    let filter = request.filter().filter_header("key2", "value2");
    assert!(filter.is_none());
}

#[test]
fn test_query() {
    use http::request::Builder;
    use crate::request::Extension;
    let request = Builder::new().uri("https://www.rust-lang.org/?one=two&three=four").body(()).unwrap();
    let filter = request.filter().filter_query("one", "two");
    assert!(filter.is_some());
    let filter = request.filter().filter_query("three", "four");
    assert!(filter.is_some());
    let filter = request.filter().filter_query("{}", "four");
    assert!(filter.is_some());
    let filter = request.filter().filter_query("one", "{}");
    assert!(filter.is_some());
    let filter = request.filter().filter_query("{}", "{}");
    assert!(filter.is_some());
    let filter = request.filter().filter_query("one", "three");
    assert!(filter.is_none());
    let filter = request.filter().filter_query("five", "six");
    assert!(filter.is_none());
}

#[test]
fn test_method() {
    use http::request::Builder;
    use http::method::Method;
    use crate::request::Extension;
    let request = Builder::new().uri("https://www.rust-lang.org/").method("POST").body(()).unwrap();
    let filter = request.filter().filter_method("POST");
    assert!(filter.is_some());
    let filter = request.filter().filter_method(Method::POST);
    assert!(filter.is_some());
    let filter = request.filter().filter_method("GET");
    assert!(filter.is_none());
}

#[test]
fn test_custom() {
    use http::request::Builder;
    use crate::request::Extension;
    let request = Builder::new().uri("https://www.rust-lang.org/").body(()).unwrap();
    let filter = request.filter().filter_custom(|_| true);
    assert!(filter.is_some());
    let filter = request.filter().filter_custom(|_| false);
    assert!(filter.is_none());
}

#[test]
fn test_scheme() {
    use http::request::Builder;
    use crate::request::Extension;
    let request = Builder::new().uri("https://www.rust-lang.org/").body(()).unwrap();
    let filter = request.filter().filter_scheme("https");
    assert!(filter.is_some());
    let filter = request.filter().filter_scheme("http");
    assert!(filter.is_none());
}

#[test]
fn test_multiple_filters(){
    use http::request::Builder;
    use crate::request::Extension;
    let request = Builder::new().uri("https://www.rust-lang.org/").method("POST").body(()).unwrap();
    let filter = request.filter().filter_scheme("https").filter_method("POST");
    assert!(filter.is_some());
    let filter = request.filter().filter_scheme("http").filter_method("get");
    assert!(filter.is_none());
}