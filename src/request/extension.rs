use http::request::Request;

pub fn query_iter(query : &str) -> impl Iterator<Item=(&str, &str)>{
    query.split('&')
        .map(|q| {
            let mut q = q.split('=').fuse();
            (q.next(), q.next())
        })
        .filter(|(key, value)| key.is_some() && value.is_some())
        .map(|(key, value)| (key.unwrap(), value.unwrap())) 
}
