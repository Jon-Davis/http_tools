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
use crate::request::{query_iter, FilterStatus, Handlers, HandlerResult};
use std::marker::PhantomData;

type ResponseHandler<'a, R> = Option<fn(&'a Request<R>) -> HandlerResult>;
pub struct Filter<'a, R, H : Handlers<R>> {
    request: &'a Request<R>,
    response : ResponseHandler<'a, R>,
    pass_throughs : u8,
    committed: bool,
    status : FilterStatus,
    handlers : PhantomData<H>,
}


impl<'a, R, H : Handlers<R>> Filter<'a, R, H>{
    /// Constructs a new Filter given a &Request, It is recommended to use request::Extension::new() or
    /// request::Extension::new_with_handlers() to construct a filter
    pub(crate) fn new(request : &'a Request<R>) -> Filter<'a, R, H> {
        Filter{
            request,
            response: None,
            status: FilterStatus::Pass,
            pass_throughs: 0,
            committed: false,
            handlers: PhantomData,
        }
    }
    /// Shorthand function to cause a failure and return self
    fn fail(mut self, status : FilterStatus) -> Self {
        self.status = status;
        self
    }

    /// Shorthand function to pass a function, wiping the previous handler
    fn pass(self) -> Self {
        self
    }

    /// Shorthand function when a passing through a failed filter is required
    fn pass_through(mut self) -> Self {
        self.pass_throughs = self.pass_throughs.saturating_add(1);
        self
    }

    pub fn on_fail(mut self, handler: fn(&'a Request<R>) -> HandlerResult) -> Self {
        if self.pass_throughs == 0 && self.status != FilterStatus::Pass {
            self.response = Some(handler);
            self.commit().pass_through()
        } else {
            self
        }
    }

    /// Handle consumes the filter, it's behavior is dependent on the state of the filter
    ///
    /// * If the filter is passing, then the closure passed as an argument will be run. As a result
    /// `handle()` will return Some(Result<Response>).
    /// * If the filter is failing, but a response handler was set through an `on_fail()` method while the
    /// filter was passing, then that function will be called to generate a response. As a result `handle()`
    /// will return Some(Result<Response>).
    /// * If the filter is failing, but the `commit()` function was called while the filter was passing, and
    /// there is no response handler set by the `on_fail` function. Then the default handler for the filter
    /// will be run. As a result `handle()` will return Some(Result<Response>).
    /// * Otherwise the filter failed before it was committed. As a result `handle()` will return None.
    ///
    /// # Example
    /// ```
    /// use bytes::Bytes;
    /// use http::{request, response, method::Method};
    /// use http_tools::request::Extension;
    ///
    /// # let request = request::Builder::new()
    /// #                    .uri("https://www.rust-lang.org/")
    /// #                    .method(Method::GET)
    /// #                    .body(Bytes::new())
    /// #                    .unwrap();
    /// // This passing filter is looking for a path of `/` and a method of GET
    /// let response = request.filter_http()
    ///                     .filter_path("/") 
    ///                     .filter_method(Method::GET)
    ///                     .handle(|_| Ok(Default::default()));
    /// // Default::default() http response returns an empty 200
    /// assert!(response.unwrap().unwrap().status() == 200);
    ///
    /// // This failing filter is looking for a path of `/` and a method of POST
    /// let response = request.filter_http()
    ///                     .filter_path("/")
    ///                     .commit()
    ///                     .filter_method(Method::POST)
    ///                     .handle(|_| Ok(Default::default()));
    /// // The default handler returns a 405 Method not found when a filter_method() fails
    /// assert!(response.unwrap().unwrap().status() == 405);
    ///
    /// // This failing filter is looking for a path of `/failed/path`
    /// let response = request.filter_http()
    ///                     .filter_path("/failed/path")
    ///                     .on_fail(|_| Ok(response::Builder::new()
    ///                         .status(123)
    ///                         .body(Bytes::new())?))
    ///                     .filter_method(Method::POST)
    ///                     .handle(|_| Ok(Default::default()));
    /// // Since the previous filter failed, the on_fail response is run by handle
    /// assert!(response.unwrap().unwrap().status() == 123);
    /// ```
    pub fn handle(self, handler: fn(&'a Request<R>) -> HandlerResult) -> Option<HandlerResult> {
        match (self.response, self.committed, self.status) {
            (Some(handler), _, _) => Some(handler(self.request)),
            (None, _, FilterStatus::Pass) => Some(handler(self.request)),
            (None, true, _) => Some(H::handle(self.request, self.status)),
            _ => None,
        }
    }

    /// Commits to returning some response if all previous filters passed. If a filter fails and no 
    /// `on_fail()` handler was specified than the default handler will be run. The `handle()` method
    /// returns a `Option<Result<Response, Error>>`. If commit is called while the filter is passing
    /// handle is guaranteed to return `Some(_)`. If the filter has already failed before commit is called
    /// then handle is guaranteed to return `None`. Other methods like `on_fail()` and `handle()` 
    /// implicitly commit the filter.no
    /// # Example
    /// ```
    /// use bytes::Bytes;
    /// use http::{request, response, method::Method};
    /// use http_tools::request::Extension;
    ///
    /// // This request has a path of `/` and a method of GET
    /// let request = request::Builder::new()
    ///                     .uri("https://www.rust-lang.org/")
    ///                     .method(Method::GET)
    ///                     .body(Bytes::new())
    ///                     .unwrap();
    ///
    /// // This filter is looking for a path of `/` and a method of POST
    /// let response = request.filter_http()
    ///                     .filter_path("/") 
    ///                     .commit() // Only commit if the path matches up
    ///                     .filter_method(Method::POST)
    ///                     .handle(|_| Ok(response::Builder::new()
    ///                         .status(200)
    ///                         .body(Bytes::from("Hello World"))?));
    ///
    /// // Because the filter committed after filter_path which evaluated to true
    /// // The response is guaranteed to be Some(), in this case we assume that a valid
    /// // response::Response object was created.
    /// let response = response.unwrap().unwrap();
    /// // The filter_method function failed, which means that the handle closure will not be
    /// // called, instead the default handler will be called, since the failure was during a
    /// // filter_method function the status will be 405 Method not Allowed.
    /// assert!(response.status() == 405);
    /// assert!(*response.body() == Bytes::new());
    /// ```
    pub fn commit(mut self) -> Self {
        if self.status == FilterStatus::Pass {
            self.committed = true;
        }
        self
    }

    /// Returns true if the filter has passed all of the checks, false if it has failed at least one
    /// # Example
    /// ```
    /// # use http::request::Builder;
    /// use http_tools::request::{Extension, Filter};
    /// 
    /// # // Request Builder found in http crate
    /// # let request = Builder::new()
    /// #                .uri("https://www.rust-lang.org/")
    /// #                .header("key", "value")
    /// #                .body(()).unwrap();
    /// 
    /// let filter = request.filter_http();
    /// assert!(filter.valid());
    /// 
    /// let filter = request.filter_http().filter_custom(|_| false);
    /// assert!(!filter.valid());
    /// ```
    pub fn valid(&self) -> bool {
        self.status == FilterStatus::Pass
    }

    /// Checks to see if the request has the specified key and value stored in a header. 
    /// # Example
    /// ```
    /// # use http::request::Builder;
    /// # use bytes::Bytes;
    /// use http_tools::request::Extension;
    /// 
    /// # // Request Builder found in http crate
    /// # let request = Builder::new()
    /// #                .uri("https://www.rust-lang.org/")
    /// #                .header("Content-Type", "application/x-www-form-urlencoded")
    /// #                .body(Bytes::new()).unwrap();
    /// let filter = request.filter_http()
    ///                 .filter_header("Content-Type", "application/x-www-form-urlencoded");
    /// 
    /// assert!(filter.valid());
    /// ```
    pub fn filter_header<V>(self, key : &str, value : V) -> Self where HeaderValue : PartialEq<V>{
        if self.status != FilterStatus::Pass { return  self.pass_through(); }
        match self.request.headers().get(key) {
            // if the header exists and the header_value is contained in value, the filter passes
            Some(header_value) if *header_value == value => self.pass(),
            // otherwise the filter fails
            _ => self.fail(FilterStatus::FailFilterHeader)
        }
    }

    /// Checks to see if the uri contains a query with the given key and value. The key and value
    /// can be either url encoded or url decoded. 
    ///
    /// **NOTE** When matching on url encoded key and values the match is verbatim. So if the key
    /// is "Hello%20World" it will not match "Hello+World". However both "Hello%20World" and "Hello+World"
    /// will match "Hello World".
    /// # Example
    /// ```
    /// use http::request::Builder;
    /// use http_tools::request::{Extension, Filter};
    /// // Request Builder found in http crate
    /// let request = Builder::new()
    ///                     .uri("https://www.rust-lang.org/?one+%2B+one=two")
    ///                     .body(()).unwrap();
    /// 
    /// 
    /// let filter = request.filter_http()
    ///         // The key and value will be url decoded
    ///         .filter_query("one + one", "two")
    ///         // The url encoded strings will match verbatim  
    ///         .filter_query("one+%2B+one", "two");
    ///         
    ///         
    /// assert!(filter.valid());
    /// ```
    pub fn filter_query(self, key : &str, value : &str) -> Self {
        if self.status != FilterStatus::Pass { return  self.pass_through(); }
        // iterate through the query
        let filter = query_iter(self.request)
            .find(|(k,_)| *k == key)
            .filter(|(_,v)| *v == value).is_some();
        
        if filter {
            self.pass()
        } else {
            self.fail(FilterStatus::FailFilterQuery)
        }
    }
    /// Checks to see if the requests path matches the specified pattern. The '{}'
    /// pattern can be used can be used to match any text between forward slashes
    /// so '/{}' will match '/any' but not '/any/more'. For matching the rest of the pattern the
    /// pattern '*' can be used so '/*' will match all paths. 
    /// # Example
    /// ```
    /// use http::request::Builder;
    /// use bytes::Bytes;
    /// use http_tools::request::{Extension, Filter};
    /// 
    /// // Request Builder found in http crate
    /// let request = Builder::new()
    ///                     .uri("https://www.rust-lang.org/var/static")
    ///                     .body(Bytes::new()).unwrap();
    /// 
    /// // this will match because the paths are an exact match
    /// let filter = request.filter_http().filter_path("/var/static");
    /// assert!(filter.valid());
    /// 
    /// // this will not match as the pattern is different
    /// let filter = request.filter_http().filter_path("/something/different");
    /// assert!(!filter.valid());
    /// 
    /// // this will match because the wildcard '{}' will match var
    /// let filter = request.filter_http().filter_path("/{}/static");
    /// assert!(filter.valid());
    /// 
    /// // this will not match as the pattern is too short
    /// let filter = request.filter_http().filter_path("/");
    /// assert!(!filter.valid());
    ///
    /// // this will not match as the pattern is too long
    /// let filter = request.filter_http().filter_path("/var/static/oops");
    /// assert!(!filter.valid());
    /// 
    /// // this will match because the '*' token means match all remaining
    /// let filter = request.filter_http().filter_path("/*");
    /// assert!(filter.valid());
    /// ```
    pub fn filter_path(self, pattern : &str) -> Self {
        if self.status != FilterStatus::Pass { return  self.pass_through(); }
        // get the path from the uri
        let path = self.request.uri().path();
        // Return if the two are exactly equal
        if pattern == path { return self.pass() }
        // create two iterators split on the forward slash for both
        // the pattern given as an argument and the actual path of 
        // the request being filtered
        let (mut split_pattern, mut split_path) = (pattern.split('/'), path.split('/'));
        loop {
            // call next on each of the iterators
            return match (split_pattern.next(), split_path.next()) {
                (Some("*"), _) => self.pass(),
                (None, Some(_)) | (Some(_), None) => self.fail(FilterStatus::FailFilterPath),
                (Some("{}"), Some(_)) => continue,
                (Some(pattern), Some(path)) if pattern == path => continue,
                (Some(_), Some(_)) => self.fail(FilterStatus::FailFilterPath),
                (None, None) => self.pass(),
            };
        }
    }
    /// Checks to see if the request has the inputted method. If using a str the method must be upper case for the match to succeed.
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
    /// let filter = request.filter_http().filter_method("GET");
    /// assert!(filter.valid());
    /// 
    /// // this will not 
    /// let filter = request.filter_http().filter_method("POST");
    /// assert!(!filter.valid());
    /// 
    /// // http::Method can also be used
    /// let filter = request.filter_http().filter_method(Method::GET);
    /// assert!(filter.valid());
    /// ```
    pub fn filter_method<T>(self, method : T) -> Self where Method : PartialEq<T>{
        if self.status != FilterStatus::Pass { return  self.pass_through(); }
        // check to see if the method is successful
        if *self.request.method() == method { 
            self.pass()
        } else {
            // otherwise calculate valid methods
            self.fail(FilterStatus::FailFilterMethod)
        }

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
    /// let filter = request.filter_http().filter_scheme("https");
    /// assert!(filter.valid());
    /// // this will not 
    /// let filter = request.filter_http().filter_scheme("http");
    /// assert!(!filter.valid());
    /// ```
    pub fn filter_scheme(self, scheme : &str) -> Self {
        if self.status != FilterStatus::Pass { return self.pass_through(); }
        // check to see if the request scheme equals the scheme argument
        if self.request.uri().scheme_str().filter(|s| *s == scheme).is_some() {
            self.pass()
        } else {
            self.fail(FilterStatus::FailFilterQuery)
        }

    }
    /// Checks to see if the request has given port
    /// # Example
    /// ```
    /// use http::request::Builder;
    /// use http_tools::request::{Extension, Filter};
    /// // Request Builder found in http crate
    /// let request = Builder::new()
    ///                     .uri("https://www.rust-lang.org:200/")
    ///                     .body(()).unwrap();
    /// 
    /// // this will match
    /// let filter = request.filter_http().filter_port(200);
    /// assert!(filter.valid());
    /// // this will not 
    /// let filter = request.filter_http().filter_port(404);
    /// assert!(!filter.valid());
    /// ```
    pub fn filter_port(self, port : u16) -> Self {
        // check to see if the request scheme equals the scheme argument
        if self.status != FilterStatus::Pass { return self.pass_through(); }
        if self.request.uri().port_u16().filter(|p| *p == port).is_some() {
            self.pass()
        } else {
            self.fail(FilterStatus::FailFilterPort)
        }
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
    /// let filter = request.filter_http().filter_custom(|req| req.extensions().get::<i32>().is_some());
    /// assert!(filter.valid());
    /// ```
    pub fn filter_custom(self, func : fn(&Request<R>) -> bool) -> Self {
        if self.status != FilterStatus::Pass { return self.pass_through(); }
        if func(self.request) {
            self.pass()
        } else {
            self.fail(FilterStatus::FailFilterCustom)
        }
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
    let filter = request.filter_http().filter_path("/");
    assert!(filter.valid());
}

#[test]
fn test_full_route() {
    use http::request::Builder;
    use crate::request::Extension;
    let request = Builder::new().uri("https://www.rust-lang.org/this/is/a/longer/path").body(()).unwrap();
    let filter = request.filter_http().filter_path("/this/is/a/longer/path");
    assert!(filter.valid());
}

#[test]
fn test_var_route() {
    use http::request::Builder;
    use crate::request::Extension;
    let request = Builder::new().uri("https://www.rust-lang.org/var/static").body(()).unwrap();
    let filter = request.filter_http().filter_path("/{}/static");
    assert!(filter.valid());
}

#[test]
fn test_partial_route() {
    use http::request::Builder;
    use crate::request::Extension;
    let request = Builder::new().uri("https://www.rust-lang.org/this/is/different").body(()).unwrap();
    let filter = request.filter_http().filter_path("/this/is");
    assert!(!filter.valid());
}

#[test]
fn test_pattern_route() {
    use http::request::Builder;
    use crate::request::Extension;
    let request = Builder::new().uri("https://www.rust-lang.org/").body(()).unwrap();
    let filter = request.filter_http().filter_path("this/is/longer");
    assert!(!filter.valid());
}

#[test]
fn test_path_route() {
    use http::request::Builder;
    use crate::request::Extension;
    let request = Builder::new().uri("https://www.rust-lang.org/this/is/longer").body(()).unwrap();
    let filter = request.filter_http().filter_path("/");
    assert!(!filter.valid());
}

#[test]
fn test_path_prefix() {
    use http::request::Builder;
    use crate::request::Extension;
    let request = Builder::new().uri("https://www.rust-lang.org/this/is/longer").body(()).unwrap();
    let filter = request.filter_http().filter_path("/*");
    assert!(filter.valid());
    let filter = request.filter_http().filter_path("/this/is/*");
    assert!(filter.valid());
    let filter = request.filter_http().filter_path("/{}/*");
    assert!(filter.valid());
    let filter = request.filter_http().filter_path("/this/is/longer/than/the/original/*");
    assert!(!filter.valid());
    let filter = request.filter_http().filter_path("/th/*");
    assert!(!filter.valid());
}

#[test]
fn test_different_route() {
    use http::request::Builder;
    use crate::request::Extension;
    let request = Builder::new().uri("https://www.rust-lang.org/this/is/different").body(()).unwrap();
    let filter = request.filter_http().filter_path("/not/even/close");
    assert!(!filter.valid());
}


#[test]
fn test_header() {
    use http::request::Builder;
    use crate::request::Extension;
    let request = Builder::new().uri("https://www.rust-lang.org/").header("key", "value").body(()).unwrap();
    let filter = request.filter_http().filter_header("key", "value");
    assert!(filter.valid());
    let filter = request.filter_http().filter_header("key", "!value");
    assert!(!filter.valid());
}


#[test]
fn test_query() {
    use http::request::Builder;
    use crate::request::Extension;
    let request = Builder::new().uri("https://www.rust-lang.org/?one=two&three=four&one%2Bone=two").body(()).unwrap();
    let filter = request.filter_http().filter_query("one", "two");
    assert!(filter.valid());
    let filter = request.filter_http().filter_query("three", "four");
    assert!(filter.valid());
    let filter = request.filter_http().filter_query("one", "three");
    assert!(!filter.valid());
    let filter = request.filter_http().filter_query("five", "six");
    assert!(!filter.valid());
    let filter = request.filter_http().filter_query("one+one", "two");
    assert!(filter.valid());
}

#[test]
fn test_method() {
    use http::request::Builder;
    use http::method::Method;
    use crate::request::Extension;
    let request = Builder::new().uri("https://www.rust-lang.org/").method("POST").body(()).unwrap();
    let filter = request.filter_http().filter_method(Method::POST);
    assert!(filter.valid());
    let filter = request.filter_http().filter_method("POST");
    assert!(filter.valid());
    let filter = request.filter_http().filter_method(Method::GET);
    assert!(!filter.valid());
}

#[test]
fn test_custom() {
    use http::request::Builder;
    use crate::request::Extension;
    let request = Builder::new().uri("https://www.rust-lang.org/").body(()).unwrap();
    let filter = request.filter_http().filter_custom(|_| true);
    assert!(filter.valid());
    let filter = request.filter_http().filter_custom(|_| false);
    assert!(!filter.valid());
}

#[test]
fn test_scheme() {
    use http::request::Builder;
    use crate::request::Extension;
    let request = Builder::new().uri("https://www.rust-lang.org/").body(()).unwrap();
    let filter = request.filter_http().filter_scheme("https");
    assert!(filter.valid());
    let filter = request.filter_http().filter_scheme("http");
    assert!(!filter.valid());
}

#[test]
fn test_multiple_filters(){
    use http::request::Builder;
    use crate::request::Extension;
    let request = Builder::new().uri("https://www.rust-lang.org/").method("POST").body(()).unwrap();
    let filter = request.filter_http().filter_scheme("https").filter_method("POST");
    assert!(filter.valid());
    let filter = request.filter_http().filter_scheme("http").filter_method("GET");
    assert!(!filter.valid());
}
 
#[test]
fn test_on_fail(){
    use http::request;
    use bytes::Bytes;
    use http::response;
    use crate::request::Extension;
    let request = request::Builder::new().uri("https://www.rust-lang.org/").body(Bytes::new()).unwrap();
    let response = request.filter_http()
                        .commit()
                        .filter_custom(|_| false)
                        .on_fail(|_req| Ok(response::Builder::new()
                            .status(400)
                            .body(Bytes::new())?))
                        .handle(|_| Ok(response::Builder::new()
                            .status(200)
                            .body(Bytes::from("Hello World"))?));
    let response = response.unwrap().unwrap();
    assert!(response.status() == 400);
}

#[test]
fn test_on_fail_first_fail(){
    use http::request;
    use bytes::Bytes;
    use http::response;
    use crate::request::Extension;
    let request = request::Builder::new().uri("https://www.rust-lang.org/").body(Bytes::new()).unwrap();
    let response = request.filter_http()
                        .commit()
                        .filter_custom(|_| false)
                        .on_fail(|_req| Ok(response::Builder::new()
                            .status(400)
                            .body(Bytes::new())?))
                        .filter_custom(|_| false)
                        .on_fail(|_req| Ok(response::Builder::new()
                            .status(401)
                            .body(Bytes::new())?))
                        .handle(|_| Ok(response::Builder::new()
                            .status(200)
                            .body(Bytes::from("Hello World"))?));
    let response = response.unwrap().unwrap();
    assert!(response.status() == 400);
}

#[test]
fn test_on_fail_first_pass(){
    use http::request;
    use bytes::Bytes;
    use http::response;
    use crate::request::Extension;
    let request = request::Builder::new().uri("https://www.rust-lang.org/").body(Bytes::new()).unwrap();
    let response = request.filter_http()
                        .commit()
                        .filter_custom(|_| true)
                        .on_fail(|_req| Ok(response::Builder::new()
                            .status(400)
                            .body(Bytes::new())?))
                        .filter_custom(|_| false)
                        .on_fail(|_req| Ok(response::Builder::new()
                            .status(401)
                            .body(Bytes::new())?))
                        .handle(|_| Ok(response::Builder::new()
                            .status(200)
                            .body(Bytes::from("Hello World"))?));
    let response = response.unwrap().unwrap();
    assert!(response.status() == 401);
}

#[test]
fn test_on_fail_first_pass_second_pass(){
    use http::request;
    use bytes::Bytes;
    use http::response;
    use crate::request::Extension;
    let request = request::Builder::new().uri("https://www.rust-lang.org/").body(Bytes::new()).unwrap();
    let response = request.filter_http()
                        .commit()
                        .filter_custom(|_| true)
                        .on_fail(|_req| Ok(response::Builder::new()
                            .status(400)
                            .body(Bytes::new())?))
                        .filter_custom(|_| true)
                        .on_fail(|_req| Ok(response::Builder::new()
                            .status(401)
                            .body(Bytes::new())?))
                        .handle(|_| Ok(response::Builder::new()
                            .status(200)
                            .body(Bytes::from("Hello World"))?));
    let response = response.unwrap().unwrap();
    assert!(response.status() == 200);
}

#[test]
fn test_on_fail_first_fail_second_pass(){
    use http::request;
    use bytes::Bytes;
    use http::response;
    use crate::request::Extension;
    let request = request::Builder::new().uri("https://www.rust-lang.org/").body(Bytes::new()).unwrap();
    let response = request.filter_http()
                        .commit()
                        .filter_custom(|_| false)
                        .on_fail(|_req| Ok(response::Builder::new()
                            .status(400)
                            .body(Bytes::new())?))
                        .filter_custom(|_| true)
                        .on_fail(|_req| Ok(response::Builder::new()
                            .status(401)
                            .body(Bytes::new())?))
                        .handle(|_| Ok(response::Builder::new()
                            .status(200)
                            .body(Bytes::from("Hello World"))?));
    let response = response.unwrap().unwrap();
    assert!(response.status() == 400);
}

#[test]
fn test_handler(){
    use http::request;
    use bytes::Bytes;
    use http::response;
    use crate::request::Extension;
    let request = request::Builder::new().uri("https://www.rust-lang.org/").body(Bytes::new()).unwrap();
    let response = request.filter_http()
                        .handle(|_| Ok(response::Builder::new()
                            .status(200)
                            .body(Bytes::from("Hello World"))?));
    let response = response.unwrap().unwrap();
    assert!(response.status() == 200);
    assert!(*response.body() == Bytes::from("Hello World"));
}

#[test]
fn test_default_handler(){
    use http::request;
    use bytes::Bytes;
    use http::response;
    use crate::request::Extension;
    let request = request::Builder::new().uri("https://www.rust-lang.org/").method(Method::GET).body(Bytes::new()).unwrap();
    let response = request.filter_http()
                        .filter_path("/")
                        .commit()
                        .filter_method(Method::POST)
                        .handle(|_| Ok(response::Builder::new()
                            .status(200)
                            .body(Bytes::from("Hello World"))?));
    let response = response.unwrap().unwrap();
    assert!(response.status() == 405);
    assert!(*response.body() == Bytes::new());
}

#[test]
fn test_failed_test(){
    use http::request;
    use bytes::Bytes;
    use http::response;
    use crate::request::Extension;
    let request = request::Builder::new().uri("https://www.rust-lang.org/").method(Method::GET).body(Bytes::new()).unwrap();
    let response = request.filter_http()
                        .filter_path("/path")
                        .commit()
                        .filter_method(Method::POST)
                        .handle(|_| Ok(response::Builder::new()
                            .status(200)
                            .body(Bytes::from("Hello World"))?));
    assert!(response.is_none());
}