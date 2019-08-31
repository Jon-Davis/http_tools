use http::request::Request;
use crate::request::query_iter;


// DDD: the reason the filter methods take in a mutable refrence to self is because
// the option<Request> implementation needs to be able to switch it's self to None
pub trait Filter<'a, R, B : 'a> {
    fn filter_header(&'a mut self, key : &str, value : &str) -> B;
    fn filter_path(&'a mut self, pattern : &str) -> B;
    fn filter_path_prefix(&'a mut self, pattern : &str) -> B;
    fn filter_method(&'a mut self, method : &str) -> B;
    fn filter_query(&'a mut self, key : &str, value : &str) -> B;
    fn filter_scheme(&'a mut self, scheme : &str) -> B;
    fn filter_custom(&'a mut self, func : fn(&Request<R>) -> bool) -> B;
}

impl<'a, R> Filter<'a, R, Option<&'a mut Self>> for Request<R>{
    fn filter_header(&'a mut self, key : &str, value : &str) -> Option<&'a mut Self>{
        let mut wrap = Some(self);
        wrap.filter_header(key, value);
        wrap
    }
    fn filter_path(&'a mut self, pattern : &str) -> Option<&'a mut Self>{
        let mut wrap = Some(self);
        wrap.filter_path(pattern);
        wrap
    }
    fn filter_path_prefix(&'a mut self, pattern : &str) -> Option<&'a mut Self>{
        let mut wrap = Some(self);
        wrap.filter_path_prefix(pattern);
        wrap
    }
    fn filter_method(&'a mut self, method : &str) -> Option<&'a mut Self>{
        let mut wrap = Some(self);
        wrap.filter_method(method);
        wrap
    }
    fn filter_query(&'a mut self, key : &str, value : &str) -> Option<&'a mut Self>{
        let mut wrap = Some(self);
        wrap.filter_query(key, value);
        wrap
    }
    fn filter_scheme(&'a mut self, scheme : &str) -> Option<&'a mut Self> {
        let mut wrap = Some(self);
        wrap.filter_scheme(scheme);
        wrap
    }
    fn filter_custom(&'a mut self, func : fn(&Request<R>) -> bool) -> Option<&'a mut Self> {
        let mut wrap = Some(self);
        wrap.filter_custom(func);
        wrap
    }
}

impl<'a, R> Filter<'a, R, &'a Self> for Option<&mut Request<R>>{

    fn filter_query(&'a mut self, key : &str, value : &str) -> &'a Self {
        if let Some(request) = self {
            let query = request.uri().query();
            if let Some(query) = query {
                let mut b = false;
                for (q_key,q_value) in query_iter(query){
                    if key == q_key && value == q_value {
                        b = true;
                        break;
                    }
                }
                if b { return self; }
            }
            *self = None;
        }
        self
    }

    fn filter_header(&'a mut self, key : &str, value : &str) -> &'a Self {
        if let Some(request) = self {
            match request.headers().get(key) {
                Some(v) if v == value => (),
                _ => *self = None,
            }
        }
        self
    }

    fn filter_path(&'a mut self, pattern : &str) -> &'a Self {
        if let Some(request) = self {
            let path = request.uri().path();
            if path == pattern {
                return self;
            } else {
                let mut split_pattern = pattern.split('/');
                let mut split_path = path.split('/');
                loop {
                    let pattern_item = split_pattern.next();
                    let path_item = split_path.next();
                    match (pattern_item, path_item) {
                        (Some(pattern), Some(path)) if pattern != "{}" && pattern != path => {
                            *self = None;
                            return self;
                        } 
                        (None, Some(_)) | (Some(_), None) => {
                                *self = None;
                                return self; 
                        }
                        (None, None) => break,
                        _ => ()
                    }
                }
            }
        }
        self
    }

    fn filter_path_prefix(&'a mut self, pattern : &str) -> &'a Self {
        if let Some(request) = self {
            let path = request.uri().path();
            if path == pattern {
                return self;
            } else {
                let mut split_pattern = pattern.split('/');
                let mut split_path = path.split('/');
                loop {
                    let pattern_item = split_pattern.next();
                    let path_item = split_path.next();
                    match (pattern_item, path_item) {
                        (Some(pattern), Some(path)) if pattern == "" && pattern != path => {
                            match split_pattern.next() {
                                Some(_) => {
                                    *self = None;
                                    return self;
                                },
                                None => return self,
                            }
                        } 
                        (Some(pattern), Some(path)) if pattern != "{}" && pattern != path => {
                            *self = None;
                            return self;
                        } 
                        (Some(_), None) => {
                                *self = None;
                                return self; 
                        }
                        (None, Some(_)) | (None, None) => break,
                        _ => ()
                    }
                }
            }
        }
        self
    }

    fn filter_method(&'a mut self, method : &str) -> &'a Self {
        if let Some(request) = self {
            if request.method() != method {
                *self = None;
            }
        }
        self
    }

    fn filter_scheme(&'a mut self, scheme : &str) -> &'a Self {
        if let Some(request) = self {
            match request.uri().scheme_str() {
                Some(s) if s == scheme => (),
                _ => *self = None,
            }
        }
        self
    }

    fn filter_custom(&'a mut self, func : fn(&Request<R>) -> bool) -> &'a Self {
        if let Some(request) = self {
            let result = func(request);
            if !result {
                *self = None;
            }
        }
       self
    }
}


#[test]
fn test_root_route() {
    use http::request::Builder;
    let mut request = Builder::new().uri("https://www.rust-lang.org/").body(()).unwrap();
    let filter = request.filter_path("/");
    assert!(filter.is_some());
}

#[test]
fn test_full_route() {
    use http::request::Builder;
    let mut request = Builder::new().uri("https://www.rust-lang.org/this/is/a/longer/path").body(()).unwrap();
    let filter = request.filter_path("/this/is/a/longer/path");
    assert!(filter.is_some());
}

#[test]
fn test_var_route() {
    use http::request::Builder;
    let mut request = Builder::new().uri("https://www.rust-lang.org/var/static").body(()).unwrap();
    let filter = request.filter_path("/{}/static");
    assert!(filter.is_some());
}

#[test]
fn test_partial_route() {
    use http::request::Builder;
    let mut request = Builder::new().uri("https://www.rust-lang.org/this/is/different").body(()).unwrap();
    let filter = request.filter_path("/this/is");
    assert!(filter.is_none());
}

#[test]
fn test_pattern_route() {
    use http::request::Builder;
    let mut request = Builder::new().uri("https://www.rust-lang.org/").body(()).unwrap();
    let filter = request.filter_path("this/is/longer");
    assert!(filter.is_none());
}

#[test]
fn test_path_route() {
    use http::request::Builder;
    let mut request = Builder::new().uri("https://www.rust-lang.org/this/is/longer").body(()).unwrap();
    let filter = request.filter_path("/");
    assert!(filter.is_none());
}

#[test]
fn test_path_prefix() {
    use http::request::Builder;
    let mut request = Builder::new().uri("https://www.rust-lang.org/this/is/longer").body(()).unwrap();
    let filter = request.filter_path_prefix("/");
    assert!(filter.is_some());
    let filter = request.filter_path_prefix("/this/is");
    assert!(filter.is_some());
    let filter = request.filter_path_prefix("/{}");
    assert!(filter.is_some());
    let filter = request.filter_path_prefix("/this/is/longer/than/the/original");
    assert!(filter.is_none());
}

#[test]
fn test_different_route() {
    use http::request::Builder;
    let mut request = Builder::new().uri("https://www.rust-lang.org/this/is/different").body(()).unwrap();
    let filter = request.filter_path("/not/even/close");
    assert!(filter.is_none());
}

#[test]
fn test_header() {
    use http::request::Builder;
    let mut request = Builder::new().uri("https://www.rust-lang.org/").header("key", "value").body(()).unwrap();
    let filter = request.filter_header("key", "value");
    assert!(filter.is_some());
    let filter = request.filter_header("key2", "value2");
    assert!(filter.is_none());
}

#[test]
fn test_query() {
    use http::request::Builder;
    let mut request = Builder::new().uri("https://www.rust-lang.org/?one=two&three=four").body(()).unwrap();
    let filter = request.filter_query("one", "two");
    assert!(filter.is_some());
    let filter = request.filter_query("three", "four");
    assert!(filter.is_some());
    let filter = request.filter_query("one", "three");
    assert!(filter.is_none());
    let filter = request.filter_query("five", "six");
    assert!(filter.is_none());
}

#[test]
fn test_method() {
    use http::request::Builder;
    let mut request = Builder::new().uri("https://www.rust-lang.org/").method("POST").body(()).unwrap();
    let filter = request.filter_method("POST");
    assert!(filter.is_some());
    let filter = request.filter_method("GET");
    assert!(filter.is_none());
}

#[test]
fn test_custom() {
    use http::request::Builder;
    let mut request = Builder::new().uri("https://www.rust-lang.org/").body(()).unwrap();
    let filter = request.filter_custom(|_| true);
    assert!(filter.is_some());
    let filter = request.filter_custom(|_| false);
    assert!(filter.is_none());
}

#[test]
fn test_scheme() {
    use http::request::Builder;
    let mut request = Builder::new().uri("https://www.rust-lang.org/").body(()).unwrap();
    let filter = request.filter_scheme("https");
    assert!(filter.is_some());
    let filter = request.filter_scheme("http");
    assert!(filter.is_none());
}