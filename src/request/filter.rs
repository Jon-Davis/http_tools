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
use http::method::Method;
use http::header::HeaderValue;
use crate::encoding::PercentEncodedStr;
use crate::request::query_iter;
use crate::interface::HttpToolsContainer;

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
pub struct Filter<'a, R> {
    request: Option<&'a Request<R>>,
    response : Option<&'a dyn Fn(&'a Request<R>) -> http::response::Builder>,
}

impl<'a, R> Filter<'a, R>{
    // Constructs a new Filter given a &Request
    pub fn new(request : &'a Request<R>) -> Filter<'a, R> {
        Filter{
            response: None,
            request: Some(request),
        }
    }

    /// Returns true if the filter has passed all of the checks, false if it has failed atleast one
    pub fn is_fufilled(&self) -> bool {
        self.request.is_some()
    }

    /// Checks to see if the request has the specified key and value. The wildcard '{}'
    /// pattern can be used to match any value, multiple values can be specified with {value1|value2},
    /// optional keys begin with '?'.
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
    /// // matches when the key is 'key' and value is 'value'
    /// let filter = request.filter().filter_header("key", "value");
    /// assert!(filter.is_fufilled());
    /// 
    /// // fails only if the key is present and the value does not match the pattern
    /// // if the key is absent this will succeed, or if the key exists and the value pattern matches
    /// let filter = request.filter().filter_header("?different_key", "different_value");
    /// assert!(filter.is_fufilled()); // still matches because different_key isn't set
    /// 
    /// // matches when the key exists
    /// let filter = request.filter().filter_header("key", "{}");
    /// assert!(filter.is_fufilled());
    /// 
    /// // matches when the key is one of the following
    /// let filter = request.filter().filter_header("key", "{foo|bar|baz|value}");
    /// assert!(filter.is_fufilled());
    /// 
    /// ```
    pub fn filter_header<T : HttpToolsContainer<HeaderValue>>(mut self, key : &str, value : T) -> Self {
        // since the filter functions can return none, we can't perform any work (and shouldn't)
        // if a previous filter invalidated the Request
        self.request = self.request.and_then(|request| {
            let (key, optional) = if key.starts_with('?') { 
                (key.get(1..).unwrap_or(""), true) 
            } else {
                (key, false)
            };

            match (request.headers().get(key), optional) {
                // if the header exists and the header_value is contained in value, the filter passes
                (Some(header_value),_) if value.http_tools_contains(header_value) => Some(request),
                // if the header was not found and the optional flag was set, the filter passes
                (None,true) => Some(request),
                // otherwise the filter failes
                _ => None
            }
        });
        self
    }
    /// Checks to see if the uri contains a query with the given key and value. The wildcard '{}'
    /// pattern can be used to match any value, multiple values can be specified with {value1|value2},
    /// optional keys begin with '?'.
    /// # Example
    /// ```
    /// use http::request::Builder;
    /// use http_tools::request::{Extension, Filter};
    /// // Request Builder found in http crate
    /// let request = Builder::new()
    ///                     .uri("https://www.rust-lang.org/?one%2Bone=two") // <-- query is decoded
    ///                     .body(()).unwrap();
    /// 
    /// // this will match as the value of one+one is two
    /// let filter = request.filter().filter_query("one+one", "two");
    /// assert!(filter.is_fufilled());
    /// 
    /// // this will match because the key of one+one exists
    /// let filter = request.filter().filter_query("one+one", "{}");
    /// assert!(filter.is_fufilled());
    /// 
    /// // this will match because the key of one+one is one of the values
    /// let filter = request.filter().filter_query("one+one", "{one|two|three}");
    /// assert!(filter.is_fufilled());
    /// 
    /// // this will NOT match because the key does not exist
    /// let filter = request.filter().filter_query("two+two", "four");
    /// assert!(!filter.is_fufilled());
    /// 
    /// // If a query key is optional the ? can be used in the key
    /// // which will match when the key is found and equal to the value pattern
    /// // or if the key is not found
    /// let filter = request.filter().filter_query("?two+two", "four");
    /// assert!(filter.is_fufilled());
    /// ```
    pub fn filter_query<T : HttpToolsContainer<PercentEncodedStr<'a>>>(mut self, key : &str, value : T) -> Self {
        // since the filter functions can return none, we can't perform any work (and shouldn't)
        // if a previous filter invalidated the Request
        self.request = self.request.and_then(|request| {
            let (key, optional) = if key.starts_with('?') { 
                (key.get(1..).unwrap_or(""), true) 
            } else {
                (key, false)
            };
            // iterate through the querys
            query_iter(&request).filter(|(k,_)| *k == key).nth(0)
                .map_or_else(|| Some(request).filter(|_| optional), 
                             |(_,v)| Some(request).filter(|_| value.http_tools_contains(&v)))
        });
        
        self
    }
    /// Checks to see if the requests path matches the specified pattern. The wildcard '{}'
    /// pattern can be used can be used to match any text between foward slashes
    /// so '/{}' will match '/any' but not '/any/more'. For mathcing the rest of the pattern the
    /// terminator '*' can be used so '/*' will match all paths. For a set of matches the pattern
    /// {First|Second|Third} can be used so /{var|val}/* will match any path starting with var or val.
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
    /// assert!(filter.is_fufilled());
    /// 
    /// // this will not match as the pattern is different
    /// let filter = request.filter().filter_path("/something/different");
    /// assert!(!filter.is_fufilled());
    /// 
    /// // this will match because the wildcard '{}' will match var
    /// let filter = request.filter().filter_path("/{}/static");
    /// assert!(filter.is_fufilled());
    /// 
    /// // this will not match as the pattern is too short
    /// let filter = request.filter().filter_path("/");
    /// assert!(!filter.is_fufilled());
    /// 
    /// // this will match because the '*' token means match all remaining
    /// let filter = request.filter().filter_path("/*");
    /// assert!(filter.is_fufilled());
    /// 
    /// // this will match because one of the options is the correct path
    /// let filter = request.filter().filter_path("/{var|val|temp}/static");
    /// assert!(filter.is_fufilled());
    /// ```
    pub fn filter_path(mut self, pattern : &str) -> Self {
        // since the filter functions can return none, we can't perform any work (and shouldn't)
        // if a previous filter invalidated the Request
        self.request = self.request.and_then(|request| {
            // get the path from the uri
            let path = request.uri().path();
            // check to see if the path and pattern equal eachother
            if path == pattern { return Some(request); }

            // create two iterators split on the forward slash for both
            // the pattern given as an argument and the actual path of 
            // the request being filtered
            let (mut split_pattern, mut split_path) = (pattern.split('/'), path.split('/'));
            loop {
                // call next on each of the iterators
                return match (split_pattern.next(), split_path.next()) {
                    // if the pattern matches equals '*' then the rest of the path is accepted
                    (Some(pattern), _) if pattern == "*" => Some(request),
                    // if they both have a result check to see if the pattern is a wildcard or they equal eachother
                    (Some(pattern), Some(path)) if !pattern.http_tools_contains(&path) => None,
                    // if the pattern or path end before one another then they are not the same length, thus not equal
                    (None, Some(_)) | (Some(_), None) => None,
                    // if both the pattern and path end at the same time then they have been equal up to this point
                    // and are assumed to be equal
                    (None, None) => Some(request),
                    // if both the pattern and path are equal or the pattern is {} continue with the loop
                    _ => continue,
                };
            }
        });
        // If the filter broke out, or self was None then return None
        self
    }
    /// Checks to see if the request has the inputed method.
    /// # Example
    /// ```
    /// use http::request::Builder;
    /// use http_tools::request::{Extension, Filter};
    /// use http::method::Method;
    /// 
    /// // Request Builder found in http crate
    /// let request = Builder::new()
    ///                     .uri("https://www.rust-lang.org/")
    ///                     .method("GET")
    ///                     .body(()).unwrap();
    /// 
    /// // this will match the method
    /// let filter = request.filter().filter_methods(&["GET"]);
    /// assert!(filter.is_fufilled());
    /// 
    /// // this will not 
    /// let filter = request.filter().filter_methods(&["POST"]);
    /// assert!(filter.is_fufilled());
    /// 
    /// // multiple methods can be specified
    /// let filter = request.filter().filter_methods(&[Method::DELETE, Method::PUT, Method::GET])
    /// assert!(filter.is_fufilled());
    /// ```
    pub fn filter_method<T : HttpToolsContainer<Method>>(mut self, methods : T) -> Self {
        // don't perform work if filter is already false
        self.request = self.request.filter(|r| methods.http_tools_contains(r.method()));
        self
    }
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
    pub fn filter_scheme<T : HttpToolsContainer<&'a str>>(mut self, scheme : T) -> Self {
        // check to see if the request scheme equals the scheme argument
        self.request = self.request.and_then(|request| {
            request.uri().scheme_str().filter(|s | scheme.http_tools_contains(&s)).map(|_| request)
        });
        // If the filter broke out, or self was None then return None
        self
    }
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
    pub fn filter_ports<T : HttpToolsContainer<u16>>(mut self, ports : T) -> Self {
        // check to see if the request scheme equals the scheme argument
        self.request = self.request.and_then(|request| {
            request.uri().port_u16().filter(|p| ports.http_tools_contains(p)).map(|_| request)
        });
        // If the filter broke out, or self was None then return None
        self
    }
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
    pub fn filter_custom(mut self, func : fn(&Request<R>) -> bool) -> Self {
        self.request = self.request.filter(|r| func(r));
        self
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
    assert!(filter.is_fufilled());
}

#[test]
fn test_full_route() {
    use http::request::Builder;
    use crate::request::Extension;
    let request = Builder::new().uri("https://www.rust-lang.org/this/is/a/longer/path").body(()).unwrap();
    let filter = request.filter().filter_path("/this/is/a/longer/path");
    assert!(filter.is_fufilled());
}

#[test]
fn test_var_route() {
    use http::request::Builder;
    use crate::request::Extension;
    let request = Builder::new().uri("https://www.rust-lang.org/var/static").body(()).unwrap();
    let filter = request.filter().filter_path("/{}/static");
    assert!(filter.is_fufilled());
}

#[test]
fn test_mul_var_route() {
    use http::request::Builder;
    use crate::request::Extension;
    let request = Builder::new().uri("https://www.rust-lang.org/var/static").body(()).unwrap();
    let filter = request.filter().filter_path("/{val|var|temp}/static");
    assert!(filter.is_fufilled());
}

#[test]
fn test_partial_route() {
    use http::request::Builder;
    use crate::request::Extension;
    let request = Builder::new().uri("https://www.rust-lang.org/this/is/different").body(()).unwrap();
    let filter = request.filter().filter_path("/this/is");
    assert!(!filter.is_fufilled());
}

#[test]
fn test_pattern_route() {
    use http::request::Builder;
    use crate::request::Extension;
    let request = Builder::new().uri("https://www.rust-lang.org/").body(()).unwrap();
    let filter = request.filter().filter_path("this/is/longer");
    assert!(!filter.is_fufilled());
}

#[test]
fn test_path_route() {
    use http::request::Builder;
    use crate::request::Extension;
    let request = Builder::new().uri("https://www.rust-lang.org/this/is/longer").body(()).unwrap();
    let filter = request.filter().filter_path("/");
    assert!(!filter.is_fufilled());
}

#[test]
fn test_path_prefix() {
    use http::request::Builder;
    use crate::request::Extension;
    let request = Builder::new().uri("https://www.rust-lang.org/this/is/longer").body(()).unwrap();
    let filter = request.filter().filter_path("/*");
    assert!(filter.is_fufilled());
    let filter = request.filter().filter_path("/this/is/*");
    assert!(filter.is_fufilled());
    let filter = request.filter().filter_path("/{}/*");
    assert!(filter.is_fufilled());
    let filter = request.filter().filter_path("/this/is/longer/than/the/original/*");
    assert!(!filter.is_fufilled());
    let filter = request.filter().filter_path("/th/*");
    assert!(!filter.is_fufilled());
}

#[test]
fn test_different_route() {
    use http::request::Builder;
    use crate::request::Extension;
    let request = Builder::new().uri("https://www.rust-lang.org/this/is/different").body(()).unwrap();
    let filter = request.filter().filter_path("/not/even/close");
    assert!(!filter.is_fufilled());
}


#[test]
fn test_header() {
    use http::request::Builder;
    use crate::request::Extension;
    let request = Builder::new().uri("https://www.rust-lang.org/").header("key", "value").body(()).unwrap();
    let filter = request.filter().filter_header("key", "value");
    assert!(filter.is_fufilled());
    let filter = request.filter().filter_header("key", "{}");
    assert!(filter.is_fufilled());
}

#[test]
fn test_header_full(){
    use http::request::Builder;
    use crate::request::Extension;
    let request = Builder::new().uri("https://www.rust-lang.org/").header("key", "value").body(()).unwrap();
    let filter = request.filter().filter_header("?key", "{foo|bar|baz|value}");
    assert!(filter.is_fufilled());
    let filter = request.filter().filter_header("?key", "{foo|bar|baz}");
    assert!(!filter.is_fufilled());
}

#[test]
fn test_header_multiple_val(){
    use http::request::Builder;
    use crate::request::Extension;
    let request = Builder::new().uri("https://www.rust-lang.org/").header("key", "value").body(()).unwrap();
    let filter = request.filter().filter_header("key", "{foo|bar|baz|value}");
    assert!(filter.is_fufilled());
    let filter = request.filter().filter_header("key", "{foo|bar|baz}");
    assert!(!filter.is_fufilled());
}

#[test]
fn test_header_optional_key(){
    use http::request::Builder;
    use crate::request::Extension;
    let request = Builder::new().uri("https://www.rust-lang.org/").header("key", "value").body(()).unwrap();
    let filter = request.filter().filter_header("?key2", "{}");
    assert!(filter.is_fufilled());
    let filter = request.filter().filter_header("key2", "value2");
    assert!(!filter.is_fufilled());
}

#[test]
fn test_query() {
    use http::request::Builder;
    use crate::request::Extension;
    let request = Builder::new().uri("https://www.rust-lang.org/?one=two&three=four&one%2Bone=two").body(()).unwrap();
    let filter = request.filter().filter_query("one", "two");
    assert!(filter.is_fufilled());
    let filter = request.filter().filter_query("three", "four");
    assert!(filter.is_fufilled());
    let filter = request.filter().filter_query("one", "{}");
    assert!(filter.is_fufilled());
    let filter = request.filter().filter_query("one", "{one|two|three}");
    assert!(filter.is_fufilled());
    let filter = request.filter().filter_query("one", "three");
    assert!(!filter.is_fufilled());
    let filter = request.filter().filter_query("?one", "three");
    assert!(!filter.is_fufilled());
    let filter = request.filter().filter_query("five", "six");
    assert!(!filter.is_fufilled());
    let filter = request.filter().filter_query("?five", "six");
    assert!(filter.is_fufilled());
    let filter = request.filter().filter_query("one+one", "two");
    assert!(filter.is_fufilled());
}

#[test]
fn test_method() {
    use http::request::Builder;
    use http::method::Method;
    use crate::request::Extension;
    let request = Builder::new().uri("https://www.rust-lang.org/").method("POST").body(()).unwrap();
    let filter = request.filter().filter_method("POST");
    assert!(filter.is_fufilled());
    let filter = request.filter().filter_method(["POST"]);
    assert!(filter.is_fufilled());
    let filter = request.filter().filter_method([Method::POST]);
    assert!(filter.is_fufilled());
    let filter = request.filter().filter_method(["GET"]);
    assert!(!filter.is_fufilled());
}

#[test]
fn test_methods() {
    use http::request::Builder;
    use http::method::Method;
    use crate::request::Extension;
    let request = Builder::new().uri("https://www.rust-lang.org/").method("POST").body(()).unwrap();
    let filter = request.filter().filter_method([Method::DELETE, Method::PUT, Method::POST]);
    assert!(filter.is_fufilled());
}

#[test]
fn test_custom() {
    use http::request::Builder;
    use crate::request::Extension;
    let request = Builder::new().uri("https://www.rust-lang.org/").body(()).unwrap();
    let filter = request.filter().filter_custom(|_| true);
    assert!(filter.is_fufilled());
    let filter = request.filter().filter_custom(|_| false);
    assert!(!filter.is_fufilled());
}

#[test]
fn test_scheme() {
    use http::request::Builder;
    use crate::request::Extension;
    let request = Builder::new().uri("https://www.rust-lang.org/").body(()).unwrap();
    let filter = request.filter().filter_scheme("https");
    assert!(filter.is_fufilled());
    let filter = request.filter().filter_scheme("http");
    assert!(!filter.is_fufilled());
}

#[test]
fn test_multiple_filters(){
    use http::request::Builder;
    use crate::request::Extension;
    let request = Builder::new().uri("https://www.rust-lang.org/").method("POST").body(()).unwrap();
    let filter = request.filter().filter_scheme("https").filter_method("POST");
    assert!(filter.is_fufilled());
    let filter = request.filter().filter_scheme("http").filter_method("get");
    assert!(!filter.is_fufilled());
}