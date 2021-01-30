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
/*          impl Filter for Option<Response>                                                    */
/*          Test Cases                                                                          */
/* ============================================================================================ */
use http::response::Response;
use http::header::HeaderValue;
use http::status::StatusCode;

const WILDCARD : &str = "{}";

/* ============================================================================================ */
/*     Filter Trait                                                                             */
/* ============================================================================================ */
/// The filter trait allows for the easy filtering of responses. They can be chained
/// together to create more complex filters. 
/// 
/// Filters were designed to be used in making an http router, but can be used in many more ways. 
/// The trait always outputs an Option<&Response>. If the option is Some then the underlying 
/// filter applies to the response, if the response is None then response did not pass the filter. 
/// The filter trait is implemented on both a Response and an Option<&Response>
/// # Syntax
/// ```
/// # use http::response::Builder;
/// # use http::status::StatusCode;
/// use http_tools::response::{Filter, ResponseExtension};
/// # let response = Builder::new()
/// #                .status(200)
/// #                .header("content-type", "application/x-www-form-urlencoded")
/// #                .header("content-length", "0")
/// #                .body(()).unwrap();
/// 
/// // given an http::response::Response
/// response
///     // Creates an Option<&Response>, each fiter returns Some(req) if it passes and None if it fails
///     .filter_http()
///     // The header has the key content-type with a value of application/x-www-form-urlencoded
///     .filter_header("content-type", "application/x-www-form-urlencoded")
///     // The {} wild card can be used to filter headers aswell
///     .filter_header("content-length", "{}")
///     // custom filters can be applied, and will be given the response and return a bool
///     .filter_custom(|req| req.extensions().get::<i32>().is_some())
///     // The status of the response can be filter on
///     .filter_status(StatusCode::OK)
///     // filters simply return std Option where Some means pass and None means failed
///     .and_then(|_response| Some("I passed the test!"));
///  ```
pub trait Filter<'a, R> {
    /// Checks to see if the response has the specified key and value. The wildcard '{}'
    /// pattern can be used in either the key or the value string. The function returns
    /// Some(response) if the header with the key and value are found or None if they are
    /// absent. 
    /// # Example
    /// ```
    /// use http::response::Builder;
    /// use http_tools::response::{ResponseExtension, Filter};
    /// 
    /// // Response Builder found in http crate
    /// let response = Builder::new()
    ///                 .header("key", "value")
    ///                 .body(()).unwrap();
    /// 
    /// // matches when the key is key and value is value
    /// let filter = response.filter_http().filter_header("key", "value");
    /// assert!(filter.is_some());
    /// 
    /// // matches when the key exists
    /// let filter = response.filter_http().filter_header("key", "{}");
    /// assert!(filter.is_some());
    /// ```
    fn filter_header<T>(self, key : &str, value : T) -> Self where T : PartialEq<HeaderValue> + PartialEq<&'static str>;
    /// filter_custom allows for a custom function filter. The filter will be given a &Response and
    /// will output a bool. if the bool is true, then function returns Some, if it is false then the
    /// function will return None
    /// # Example
    /// ```
    /// use http::response::Builder;
    /// use http_tools::response::{ResponseExtension, Filter};
    /// // Response Builder found in http crate
    /// let response = Builder::new()
    ///                     .extension(-1i32)
    ///                     .body(()).unwrap();
    /// 
    /// // this will match as the response has an extension of type i32
    /// let filter = response.filter_http().filter_custom(|req| req.extensions().get::<i32>().is_some());
    /// assert!(filter.is_some());
    /// ```
    fn filter_custom(self, func : fn(&Response<R>) -> bool) -> Self;
    /// filter_status checks to see if the status of the Response is equal to the response status.
    /// The filter will return Some(&Response) if the status codes are equal and None otherwise. If 
    /// filtering over a wider variety of errors use the filter_status_success, filter_status_client_error,
    /// filter_status_server_error, filter_status_redirection and filter_status_informational
    /// # Example
    /// ```
    /// use http::response::Builder;
    /// use http::status::StatusCode;
    /// use http_tools::response::{ResponseExtension, Filter};
    /// // Response Builder found in http crate
    /// let response = Builder::new()
    ///                     .status(200)
    ///                     .body(()).unwrap();
    /// 
    /// // this will match as the response has an extension of type i32
    /// let filter = response.filter_http().filter_status(StatusCode::OK);
    /// assert!(filter.is_some());
    /// ```
    fn filter_status<T>(self, status : T) -> Self where StatusCode : PartialEq<T>;
}

/* ============================================================================================ */
/*     impl Filter for Option<Response>                                                         */
/* ============================================================================================ */

// The filter trait implentation for Option<&Response> does the actuall filtering
// It takes in an Option<&Response> and outputs an Option<&Response> in order to allow
// for the chaining of multiple filters. If a filter function returns Some that means
// that the response passed through the filter, if a filter function returns None that
// means the response did not pass the filter.
impl<'a, R> Filter<'a, R> for Option<&Response<R>>{
    // The filter_header function for Option<&Response> first checks to see that the value
    // of self is Some. Then it checks the key, if the key is a wild card then the values
    // will need to be iterated through to check to see if they match, if the key is not
    // a wild card then we can call the get function on the Responses HeaderMap for the key.
    fn filter_header<T>(self, key : &str, value : T) -> Self where T : PartialEq<HeaderValue> + PartialEq<&'static str> {
        // since the filter functions can return none, we can't perform any work (and shouldn't)
        // if a previous filter invalidated the Response
        if let Some(response) = self {
            // Check to see if the key is the wildcard token of '{}'
            if key == WILDCARD {
                // retrieve the headers map
                let map = response.headers();
                // If the value is {} and there are entries in the header map
                // the return Some response as any value would match
                if value == WILDCARD && !map.is_empty() {
                    return Some(response);
                }
                // Iterate through the different values to see if any values
                // match the inputed value
                for v in map.values() {
                    // if the values match return Some
                    if value == *v {
                        return Some(response);
                    }
                }
            } else {
                // Get the key and check if it's value is equal to the inputed value
                // otherwise fall through to the end and return None
                match response.headers().get(key) {
                    Some(v) if value == *v || value == WILDCARD => return Some(response),
                    _ => (),
                }
            }
        }
        // If the filter broke out, or self was None then return None
        None
    }
    // The filter_scheme function for Option<&Response> first checks to see that the value of
    // self is Some, then checks to see if the response scheme is equal to the inputed scheme.
    fn filter_custom(self, func : fn(&Response<R>) -> bool) -> Self {
        if let Some(response) = self {
            let result = func(response);
            if result {
                return self;
            }
        }
       None
    }
    // The filter_status function for Option<&Response> checks to see if the given status
    // is equal to the response status. The value will be Some if they are equal and None
    // if they are not
    fn filter_status<T>(self, status : T) -> Self where StatusCode : PartialEq<T>{
        if let Some(response) = self {
            if response.status() == status {
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
fn test_header() {
    use http::response::Builder;
    use crate::response::ResponseExtension;
    use http::header::HeaderValue;
    let mut header = HeaderValue::from_str("value").unwrap();
    header.set_sensitive(true);
    let response = Builder::new().header("key", header).body(()).unwrap();
    let filter = response.filter_http().filter_header("key", "value");
    assert!(filter.is_some());
    let filter = response.filter_http().filter_header("key", "{}");
    assert!(filter.is_some());
    let filter = response.filter_http().filter_header("{}", "value");
    assert!(filter.is_some());
    let header = HeaderValue::from_str("value").unwrap();
    let filter = response.filter_http().filter_header("{}", header);
    assert!(filter.is_some());
    let filter = response.filter_http().filter_header("{}", "{}");
    assert!(filter.is_some());
    let filter = response.filter_http().filter_header("key2", "value2");
    assert!(filter.is_none());
}

#[test]
fn test_custom() {
    use http::response::Builder;
    use crate::response::ResponseExtension;
    let response = Builder::new().body(()).unwrap();
    let filter = response.filter_http().filter_custom(|_| true);
    assert!(filter.is_some());
    let filter = response.filter_http().filter_custom(|_| false);
    assert!(filter.is_none());
}

#[test]
fn test_status() {
    use http::response::Builder;
    use crate::response::ResponseExtension;
    let response = Builder::new().status(StatusCode::OK).body(()).unwrap();
    let filter = response.filter_http().filter_status(StatusCode::OK);
    assert!(filter.is_some());
    let filter = response.filter_http().filter_status(200);
    assert!(filter.is_some());
    let filter = response.filter_http().filter_status(500);
    assert!(filter.is_none());
    let filter = response.filter_http().filter_status(1000);
    assert!(filter.is_none());
}