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
/*          Filter Struct                                                                       */
/*          impl Filter                                                                         */
/*          Test Cases                                                                          */
/* ============================================================================================ */
use bytes::Bytes;
use http::{Response, request::Request};
use http::method::Method;
use http::header::HeaderValue;
use std::future::Future;
use anyhow::Result;
use crate::{request::{query_iter, FilterError}};

/// Convenience type, returned by several Filter Methods such as `handle()`, `async_handle()`, `on_fail()`, and `set_error_handler()`.
pub type HandlerResult = Result<Response<Bytes>>;

/// Wraps a `&http::Request` and allows for the filtering of requests, as well as calling handler functions to process the Request.
pub struct Filter<'a, R> {
    request: &'a Request<R>,
    error_handler : Option<fn(&Request<R>, FilterError) -> HandlerResult>,
    pass_throughs : u8,
    committed: bool,
    error : Option<FilterError>,
}


impl<'a, R> Filter<'a, R>{
    /// Constructs a new Filter given a &Request, It is recommended to use request::RequestExtension::new() or
    /// request::RequestExtension::new_with_handlers() to construct a filter
    pub(crate) fn new(request : &'a Request<R>) -> Filter<'a, R> {
        Filter{
            request,
            error_handler: None,
            error: None,
            pass_throughs: 0,
            committed: false,
        }
    }

    /// Shorthand function to cause a failure and return self
    fn fail(mut self, status : FilterError) -> Self {
        self.error = Some(status);
        self
    }

    /// Shorthand function to pass a function
    fn pass(self) -> Self {
        self
    }

    /// Shorthand function when a passing through a failed filter is required
    fn pass_through(mut self) -> Self {
        self.pass_throughs = self.pass_throughs.saturating_add(1);
        self
    }

    /// Handle consumes the filter, it's behavior is dependent on the state of the filter
    ///
    /// * If the filter is passing, then the closure passed as an argument will be run. As a result
    /// `handle()` will return Ok(HandlerResult).
    /// * If a committed filter is failing, but a response handler was set through the `on_fail()` or `set_error_handler()`
    // methods, then the error handler function will be called to generate a response. As a result `handle()`
    /// will return `Ok(HandlerResult)`.
    /// * f a committed filter is failing, but no response handler was set through the `on_fail()` or `set_error_handler()`
    // methods, then the `default_error_handler` function will be called to generate a response. As a result `handle()`
    /// will return `Ok(HandlerResult)`.
    /// * Otherwise the filter failed before it was committed. As a result `handle()` will return Err.
    ///
    /// # Example
    /// ```
    /// use bytes::Bytes;
    /// use http::{request, response, method::Method};
    /// use http_tools::request::RequestExtension;
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
    /// ```
    pub fn handle(self, handler: fn(&'a Request<R>) -> HandlerResult) -> Result<HandlerResult, FilterError> {
        match (self.error_handler, self.committed, self.error) {
            (_, _, None) => Ok(handler(self.request)),
            (Some(response), true, Some(err)) => Ok((response)(self.request, err)),
            (None, true, Some(err)) => Ok(Self::default_error_handler(self.request, err)),
            (_, false, Some(err))=> Err(err),
        }
    }

    /// An Async Handle that consumes the filter, it's behavior is dependent on the state of the filter
    ///
    /// * If the filter is passing, then the closure passed as an argument will be run. As a result
    /// `async_handle()` will return `impl Future<Output=Ok(HandlerResult)>`.
    /// * If a committed filter is failing, but a response handler was set through the `on_fail()` or `set_error_handler()`
    // methods, then the error handler function will be called to generate a response. As a result `async_handle()`
    /// will return `impl Future<Output=Ok(HandlerResult)>`.
    /// * f a committed filter is failing, but no response handler was set through the `on_fail()` or `set_error_handler()`
    // methods, then the `default_error_handler` function will be called to generate a response. As a result `async_handle()`
    /// will return `impl Future<Output=Ok(HandlerResult)>``.
    /// * Otherwise the filter failed before it was committed. As a result `handle()` will return Err.
    ///
    /// # Example
    /// ```
    /// use bytes::Bytes;
    /// use http::{request, response, method::Method};
    /// use http_tools::request::RequestExtension;
    /// use futures::executor::block_on; 
    ///
    /// block_on(async {
    /// # let request = request::Builder::new()
    /// #                    .uri("https://www.rust-lang.org/")
    /// #                    .method(Method::GET)
    /// #                    .body(Bytes::new())
    /// #                    .unwrap();
    ///     // This passing filter is looking for a path of `/` and a method of GET
    ///     let response = request.filter_http()
    ///                     .filter_path("/") 
    ///                     .filter_method(Method::GET)
    ///                     .async_handle(|_| async { 
    ///                         Ok(Default::default())
    ///                     }).await;
    ///     // Default::default() http response returns an empty 200
    ///     assert!(response.unwrap().unwrap().status() == 200);
    ///
    ///     // This failing filter is looking for a path of `/` and a method of POST
    ///     let response = request.filter_http()
    ///                     .filter_path("/")
    ///                     .commit()
    ///                     .filter_method(Method::POST)
    ///                     .async_handle(|_| async { 
    ///                         Ok(Default::default())
    ///                     }).await;    
    ///     // The default handler returns a 405 Method not found when a filter_method() fails
    ///     assert!(response.unwrap().unwrap().status() == 405);
    /// });
    /// ```
    pub async fn async_handle<F>(self, handler: fn(&'a Request<R>) -> F) -> Result<HandlerResult, FilterError> 
    where F : Future<Output=HandlerResult> {
        match (self.error_handler, self.committed, self.error) {
            (_, _, None) => Ok(handler(self.request).await),
            (Some(response), true, Some(err)) => Ok((response)(self.request, err)),
            (None, true, Some(err)) => Ok(Self::default_error_handler(self.request, err)),
            (_, false, Some(err))=> Err(err),
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
    /// use http_tools::request::RequestExtension;
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
        if self.error.is_none() {
            self.committed = true;
        }
        self
    }

    /// Provides a handler that will be run in the event that the previous filter caused a failure.
    /// The handler passed to this function will be called when the handler() or async_handle() 
    /// functions are called. If the previous filter caused a failure, than the on_fail() method
    /// will commit (see commit()) to returning the passed handler, however if the previous filter
    /// passed, or if the Filter was already failing prior to the previous filter, than the method 
    /// will not commit.
    /// # Example
    /// ```
    /// use bytes::Bytes;
    /// use http::{request, method::Method, StatusCode, response::Response};
    /// use http_tools::{request::RequestExtension, response::ResponseExtension};
    ///
    /// # let request = request::Builder::new()
    /// #                    .uri("https://www.rust-lang.org/")
    /// #                    .method(Method::GET)
    /// #                    .body(Bytes::new())
    /// #                    .unwrap();
    /// // This failing filter is looking for a path of `/failed/path`
    /// let response = request.filter_http()
    ///                     .filter_path("/failed/path")
    ///                     .on_fail(|_req, _status| Ok(Response::<Bytes>::from_status(StatusCode::IM_A_TEAPOT)))
    ///                     .filter_method(Method::POST)
    ///                     .handle(|_| Ok(Default::default()));
    /// // Since the previous filter failed, the on_fail response is run by handle
    /// assert!(response.unwrap().unwrap().status() == StatusCode::IM_A_TEAPOT);
    /// ```
    pub fn on_fail(mut self, handler: fn(&Request<R>, FilterError) -> HandlerResult) -> Self {
        if self.pass_throughs == 0 && self.error.is_some(){
            self.error_handler = Some(handler);
            self.committed = true;
            self.pass_through()
        } else {
            self
        }
    }

    /// The default error handler, used when a committed Filter failed and needs to return a response. The default
    /// error handler is made available to make it easier to create custom error handlers using the `.set_error_handler()`
    /// method.
    /// ```
    /// use bytes::Bytes;
    /// use http::{request, method::Method, StatusCode, response::Response};
    /// use http_tools::{request::{RequestExtension, Filter, FilterError}, response::ResponseExtension};
    ///
    /// # let request = request::Builder::new()
    /// #                    .uri("https://www.rust-lang.org/")
    /// #                    .method(Method::GET)
    /// #                    .body(Bytes::new())
    /// #                    .unwrap();
    /// // This failing filter is looking for a path of `/failed/path`
    /// let response = request.filter_http()
    ///                     .commit()
    ///                     .filter_custom(|_| false)
    ///                     .filter_method(Method::POST)
    ///                     .set_error_handler(|req, status| match status {
    ///                         FilterError::FailFilterCustom => Ok(Response::<Bytes>::from_status(StatusCode::IM_A_TEAPOT)),
    ///                         _ => Filter::default_error_handler(req, status),
    ///                     })
    ///                     .handle(|_| Ok(Default::default()));
    /// // Since the previous filter failed, the on_fail response is run by handle
    /// assert!(response.unwrap().unwrap().status() == StatusCode::IM_A_TEAPOT);
    /// ```
    pub fn default_error_handler(req : &Request<R>, err : FilterError) -> HandlerResult {
        use http::response::Builder;
        match err {
            FilterError::FailFilterPath => Ok(Builder::new()
                    .status(404)
                    .version(req.version())
                    .header("Content-Length", 0)
                    .body(Bytes::new())?),
            FilterError::FailFilterMethod => Ok(Builder::new()
                    .status(405)
                    .version(req.version())
                    .header("Content-Length", 0)
                    .body(Bytes::new())?),
            _ => Ok(Builder::new()
                    .status(400)
                    .version(req.version())
                    .header("Content-Length", 0)
                    .body(Bytes::new())?),
        }
    }

    /// Filters provide a basic error handler that will generate a http::Response given the state
    /// of a committed filter. For example the default Error Handler will generate a http::Response with a status
    /// code of 405 if the filter failed due to a .filter_method() call. set_error_handler() will override
    /// the default error handlers.
    /// ```
    /// use bytes::Bytes;
    /// use http::{request, method::Method, StatusCode, response::Response};
    /// use http_tools::{request::{RequestExtension, Filter, FilterError}, response::ResponseExtension};
    ///
    /// # let request = request::Builder::new()
    /// #                    .uri("https://www.rust-lang.org/")
    /// #                    .method(Method::GET)
    /// #                    .body(Bytes::new())
    /// #                    .unwrap();
    /// // This failing filter is looking for a path of `/failed/path`
    /// let response = request.filter_http()
    ///                     .commit()
    ///                     .filter_custom(|_| false)
    ///                     .filter_method(Method::POST)
    ///                     .set_error_handler(|req, status| match status {
    ///                         FilterError::FailFilterCustom => Ok(Response::<Bytes>::from_status(StatusCode::IM_A_TEAPOT)),
    ///                         _ => Filter::default_error_handler(req, status),
    ///                     })
    ///                     .handle(|_| Ok(Default::default()));
    /// // Since the previous filter failed, the on_fail response is run by handle
    /// assert!(response.unwrap().unwrap().status() == StatusCode::IM_A_TEAPOT);
    /// ```
    pub fn set_error_handler(mut self, handler: fn(&Request<R>, FilterError) -> HandlerResult) -> Self {
        self.error_handler = self.error_handler.or(Some(handler));
        self
    }

    /// Returns true if the filter has passed all of the checks, false if it has failed at least one
    /// # Example
    /// ```
    /// # use http::request::Builder;
    /// use http_tools::request::{RequestExtension, Filter};
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
        self.error.is_none()
    }

    /// Checks to see if the request has the specified key and value stored in a header. 
    /// # Example
    /// ```
    /// # use http::request::Builder;
    /// # use bytes::Bytes;
    /// use http_tools::request::RequestExtension;
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
        if self.error.is_some() { return  self.pass_through(); }
        match self.request.headers().get(key) {
            // if the header exists and the header_value is contained in value, the filter passes
            Some(header_value) if *header_value == value => self.pass(),
            // otherwise the filter fails
            _ => self.fail(FilterError::FailFilterHeader)
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
    /// use http_tools::request::{RequestExtension, Filter};
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
        if self.error.is_some() { return  self.pass_through(); }
        // iterate through the query
        let filter = query_iter(self.request)
            .find(|(k,_)| *k == key)
            .filter(|(_,v)| *v == value).is_some();
        
        if filter {
            self.pass()
        } else {
            self.fail(FilterError::FailFilterQuery)
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
    /// use http_tools::request::{RequestExtension, Filter};
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
        if self.error.is_some() { return  self.pass_through(); }
        // get the path from the uri
        let path = self.request.uri().path();
        // create two iterators split on the forward slash for both
        // the pattern given as an argument and the actual path of 
        // the request being filtered
        let (mut split_pattern, mut split_path) = (pattern.split('/'), path.split('/'));
        loop {
            // call next on each of the iterators
            return match (split_pattern.next(), split_path.next()) {
                (Some("*"), _) => self.pass(),
                (Some(pattern), Some(path)) if pattern == path || pattern == "{}" => continue,
                (None, None) => self.pass(),
                _ => self.fail(FilterError::FailFilterPath),
            };
        }
    }

    /// Checks to see if the request has the inputted method. If using a str the method must be upper case for the match to succeed.
    /// # Example
    /// ```
    /// use http::request::Builder;
    /// use http_tools::request::{RequestExtension, Filter};
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
        if self.error.is_some() { return  self.pass_through(); }
        // check to see if the method is successful
        if *self.request.method() == method { 
            self.pass()
        } else {
            // otherwise calculate valid methods
            self.fail(FilterError::FailFilterMethod)
        }

    }

    /// Checks to see if the request has given scheme
    /// # Example
    /// ```
    /// use http::request::Builder;
    /// use http_tools::request::{RequestExtension, Filter};
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
        if self.error.is_some() { return self.pass_through(); }
        // check to see if the request scheme equals the scheme argument
        if self.request.uri().scheme_str().filter(|s| *s == scheme).is_some() {
            self.pass()
        } else {
            self.fail(FilterError::FailFilterQuery)
        }

    }

    /// Checks to see if the request has given port
    /// # Example
    /// ```
    /// use http::request::Builder;
    /// use http_tools::request::{RequestExtension, Filter};
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
        if self.error.is_some() { return self.pass_through(); }
        if self.request.uri().port_u16().filter(|p| *p == port).is_some() {
            self.pass()
        } else {
            self.fail(FilterError::FailFilterPort)
        }
    }
    
    /// filter_custom allows for a custom function filter. The filter will be given a &Request and
    /// will output a bool. if the bool is true, then function returns Some, if it is false then the
    /// function will return None
    /// # Example
    /// ```
    /// use http::request::Builder;
    /// use http_tools::request::{RequestExtension, Filter};
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
        if self.error.is_some() { return self.pass_through(); }
        if func(self.request) {
            self.pass()
        } else {
            self.fail(FilterError::FailFilterCustom)
        }
    }
}

/* ============================================================================================ */
/*     Test Cases                                                                               */
/* ============================================================================================ */
#[test]
fn test_root_route() {
    use http::request::Builder;
    use crate::request::RequestExtension;
    let request = Builder::new().uri("https://www.rust-lang.org/").body(()).unwrap();
    let filter = request.filter_http().filter_path("/");
    assert!(filter.valid());
}

#[test]
fn test_full_route() {
    use http::request::Builder;
    use crate::request::RequestExtension;
    let request = Builder::new().uri("https://www.rust-lang.org/this/is/a/longer/path").body(()).unwrap();
    let filter = request.filter_http().filter_path("/this/is/a/longer/path");
    assert!(filter.valid());
}

#[test]
fn test_var_route() {
    use http::request::Builder;
    use crate::request::RequestExtension;
    let request = Builder::new().uri("https://www.rust-lang.org/var/static").body(()).unwrap();
    let filter = request.filter_http().filter_path("/{}/static");
    assert!(filter.valid());
}

#[test]
fn test_partial_route() {
    use http::request::Builder;
    use crate::request::RequestExtension;
    let request = Builder::new().uri("https://www.rust-lang.org/this/is/different").body(()).unwrap();
    let filter = request.filter_http().filter_path("/this/is");
    assert!(!filter.valid());
}

#[test]
fn test_pattern_route() {
    use http::request::Builder;
    use crate::request::RequestExtension;
    let request = Builder::new().uri("https://www.rust-lang.org/").body(()).unwrap();
    let filter = request.filter_http().filter_path("this/is/longer");
    assert!(!filter.valid());
}

#[test]
fn test_path_route() {
    use http::request::Builder;
    use crate::request::RequestExtension;
    let request = Builder::new().uri("https://www.rust-lang.org/this/is/longer").body(()).unwrap();
    let filter = request.filter_http().filter_path("/");
    assert!(!filter.valid());
}

#[test]
fn test_path_prefix() {
    use http::request::Builder;
    use crate::request::RequestExtension;
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
    use crate::request::RequestExtension;
    let request = Builder::new().uri("https://www.rust-lang.org/this/is/different").body(()).unwrap();
    let filter = request.filter_http().filter_path("/not/even/close");
    assert!(!filter.valid());
}


#[test]
fn test_header() {
    use http::request::Builder;
    use crate::request::RequestExtension;
    let request = Builder::new().uri("https://www.rust-lang.org/").header("key", "value").body(()).unwrap();
    let filter = request.filter_http().filter_header("key", "value");
    assert!(filter.valid());
    let filter = request.filter_http().filter_header("key", "!value");
    assert!(!filter.valid());
}


#[test]
fn test_query() {
    use http::request::Builder;
    use crate::request::RequestExtension;
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
    use crate::request::RequestExtension;
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
    use crate::request::RequestExtension;
    let request = Builder::new().uri("https://www.rust-lang.org/").body(()).unwrap();
    let filter = request.filter_http().filter_custom(|_| true);
    assert!(filter.valid());
    let filter = request.filter_http().filter_custom(|_| false);
    assert!(!filter.valid());
}

#[test]
fn test_scheme() {
    use http::request::Builder;
    use crate::request::RequestExtension;
    let request = Builder::new().uri("https://www.rust-lang.org/").body(()).unwrap();
    let filter = request.filter_http().filter_scheme("https");
    assert!(filter.valid());
    let filter = request.filter_http().filter_scheme("http");
    assert!(!filter.valid());
}

#[test]
fn test_multiple_filters(){
    use http::request::Builder;
    use crate::request::RequestExtension;
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
    use crate::request::RequestExtension;
    let request = request::Builder::new().uri("https://www.rust-lang.org/").body(Bytes::new()).unwrap();
    let response = request.filter_http()
                        .commit()
                        .filter_custom(|_| false)
                        .on_fail(|_, _| Ok(response::Builder::new()
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
    use crate::request::RequestExtension;
    let request = request::Builder::new().uri("https://www.rust-lang.org/").body(Bytes::new()).unwrap();
    let response = request.filter_http()
                        .commit()
                        .filter_custom(|_| false)
                        .on_fail(|_, _| Ok(response::Builder::new()
                            .status(400)
                            .body(Bytes::new())?))
                        .filter_custom(|_| false)
                        .on_fail(|_, _| Ok(response::Builder::new()
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
    use crate::request::RequestExtension;
    let request = request::Builder::new().uri("https://www.rust-lang.org/").body(Bytes::new()).unwrap();
    let response = request.filter_http()
                        .commit()
                        .filter_custom(|_| true)
                        .on_fail(|_, _| Ok(response::Builder::new()
                            .status(400)
                            .body(Bytes::new())?))
                        .filter_custom(|_| false)
                        .on_fail(|_, _| Ok(response::Builder::new()
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
    use crate::request::RequestExtension;
    let request = request::Builder::new().uri("https://www.rust-lang.org/").body(Bytes::new()).unwrap();
    let response = request.filter_http()
                        .commit()
                        .filter_custom(|_| true)
                        .on_fail(|_, _| Ok(response::Builder::new()
                            .status(400)
                            .body(Bytes::new())?))
                        .filter_custom(|_| true)
                        .on_fail(|_, _| Ok(response::Builder::new()
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
    use crate::request::RequestExtension;
    let request = request::Builder::new().uri("https://www.rust-lang.org/").body(Bytes::new()).unwrap();
    let response = request.filter_http()
                        .commit()
                        .filter_custom(|_| false)
                        .on_fail(|_, _| Ok(response::Builder::new()
                            .status(400)
                            .body(Bytes::new())?))
                        .filter_custom(|_| true)
                        .on_fail(|_, _| Ok(response::Builder::new()
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
    use crate::request::RequestExtension;
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
    use crate::request::RequestExtension;
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
    use crate::request::RequestExtension;
    let request = request::Builder::new().uri("https://www.rust-lang.org/").method(Method::GET).body(Bytes::new()).unwrap();
    let response = request.filter_http()
                        .filter_path("/path")
                        .commit()
                        .filter_method(Method::POST)
                        .handle(|_| Ok(response::Builder::new()
                            .status(200)
                            .body(Bytes::from("Hello World"))?));
    assert!(response.is_err());
}

#[test]
fn test_failed_twice_test(){
    use http::request;
    use bytes::Bytes;
    use http::response;
    use crate::request::RequestExtension;
    let request = request::Builder::new().uri("https://www.rust-lang.org/").method(Method::GET).body(Bytes::new()).unwrap();
    let response = request.filter_http()
                        .filter_path("/path")
                        .commit()
                        .filter_method(Method::POST)
                        .handle(|_| Ok(response::Builder::new()
                            .status(200)
                            .body(Bytes::from("Hello World"))?));
    assert!(response.is_err());
}

#[test]
fn test_async_handler(){
    use http::request;
    use bytes::Bytes;
    use http::response;
    use crate::request::RequestExtension;
    use futures::executor::block_on;
    let request = request::Builder::new().uri("https://www.rust-lang.org/")
        .method(Method::GET)
        .body(Bytes::new())
        .unwrap();
    let response = block_on(request.filter_http()
                            .filter_path("/failed/path")
                            .on_fail(|_, _| Ok(response::Builder::new()
                                .status(123)
                                .body(Bytes::new())?))
                            .filter_method(Method::POST)
                            .async_handle(|_| async { 
                                Ok(Default::default())
                            }));
    assert!(response.unwrap().unwrap().status() == 123);
}

#[test]
fn test_async_handler_returns(){
    use http::{request, StatusCode};
    use bytes::Bytes;
    use http::response::{Response, Builder};
    use crate::request::RequestExtension;
    use crate::response::ResponseExtension;
    use futures::future::TryFutureExt;
    use futures::executor::block_on;
    let request = request::Builder::new().uri("https://www.rust-lang.org/item/grapes")
        .method(Method::GET)
        .body(Bytes::new())
        .unwrap();

    block_on(async {
        // GET /item/{:string} -> Got any {:string}
        let sv1 = request.filter_http()
                    .filter_path("/item/{}") 
                    .async_handle(|req| async move {
                        let input = req.get_path_var(1).unwrap();
                        let output = format!("Got any {}", input);
                        Ok(Builder::new().body(Bytes::from(output)).unwrap())
                    });
        
        // GET /hello/{:string} -> Hello {:string}
        let sv2  = |_| {
            request.filter_http()
                .filter_path("/hello/{}") 
                .async_handle(|_| async move {
                    unreachable!();
                })
            };

        // Lazy evaluate paths, set default 404 and 500 errors
        let response = sv1
            .or_else(sv2).await
            .unwrap_or_else(|_| Ok(Response::<Bytes>::from_status(StatusCode::NOT_FOUND)))
            .unwrap_or_else(Response::<Bytes>::from_error);
        // Got any grapes?
        assert!(response.body() == "Got any grapes");
    });
}

#[test]
fn test_error_to_response(){
    use http::{request, StatusCode};
    use bytes::Bytes;
    use http::response::{Response};
    use crate::request::RequestExtension;
    use crate::response::ResponseExtension;
    use futures::executor::block_on;
    use anyhow::Context;
    let request = request::Builder::new().uri("https://www.rust-lang.org/")
        .method(Method::GET)
        .body(Bytes::new())
        .unwrap();

    block_on(async {
        // GET /item/{:string} -> Got any {:string}
        let sv1 = request.filter_http()
                    .filter_path("/") 
                    .async_handle(|_| async move {
                        u8::from_str_radix("abc", 10)
                            .context(StatusCode::IM_A_TEAPOT)
                            .context("Short and spout!")?;
                        unreachable!();
                    });
        
        // Lazy evaluate paths, set default 404 and 500 errors
        let response = sv1.await
            .unwrap_or_else(|_| Ok(Response::<Bytes>::from_status(StatusCode::NOT_FOUND)))
            .unwrap_or_else(Response::<Bytes>::from_error);

        assert!(response.status() == StatusCode::IM_A_TEAPOT);
        assert!(response.body() == "Short and spout!");
    });
}

#[test]
fn test_custom_handlers(){
    use bytes::Bytes;
    use http::{request, method::Method, StatusCode, response::Response};
    use crate::{request::{RequestExtension, Filter, FilterError}, response::ResponseExtension};

    let request = request::Builder::new()
                        .uri("https://www.rust-lang.org/")
                        .method(Method::GET)
                        .body(Bytes::new())
                        .unwrap();
    // This failing filter is looking for a path of `/failed/path`
    let response = request.filter_http()
                        .commit()
                        .filter_custom(|_| false)
                        .filter_method(Method::POST)
                        .set_error_handler(|req, status| match status {
                            FilterError::FailFilterCustom => Ok(Response::<Bytes>::from_status(StatusCode::IM_A_TEAPOT)),
                            _ => Filter::default_error_handler(req, status),
                        })
                        .handle(|_| Ok(Default::default()));
    // Since the previous filter failed, the on_fail response is run by handle
    assert!(response.unwrap().unwrap().status() == StatusCode::IM_A_TEAPOT);
}