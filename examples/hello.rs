use std::convert::Infallible;
use std::net::SocketAddr;
use bytes::Bytes;
use http::StatusCode;
use anyhow::Result;
use hyper::{Body, Request, Response, Server};
use hyper::service::{make_service_fn, service_fn};
use http_tools::{request::{RequestExtension, HandlerResult, FilterError}, response::ResponseExtension};

// The controller will filter the request, and if all filters pass, it will execute a handler
// For this example the handler processes the request for relaxant information, passes that information
// to the impl service, then takes the output of the impl service and converts it to a http::response.
// The controller will run the hello_world service, whenever a GET /hello/{} request is received.
async fn hello_world_controller(req : &Request<Body>) -> Result<HandlerResult, FilterError> {
    req.filter_http()
        .filter_path("/hello/{}")
        .filter_method("GET")
        .async_handle(|req| async move {
            let input = req.get_path_var(1).unwrap_or("");
            let output = hello_world_impl(input).await;
            Ok(Response::builder().body(Bytes::from(output))?)
        }).await
}

// The impl of the above service, it simply returns Hello World!
async fn hello_world_impl(name : &str) -> String {
    format!("Hello {}!", name)
}

// The mux is the caller of the controllers, but for this example we only have one.
// The mux will call each controller one at a time, if none off the controllers 
// processed the request, than the mux will create a default 404 Not Found response.
// If one of the controllers did process the request, but that process resulted in an
// error, than the mux will generate a response from the error. Otherwise the
async fn mux(req : Request<Body>) -> Result<Response<Body>, Infallible> {
    // Run the hello_world service
    Ok(hello_world_controller(&req)
        // wait for the selected service to finish processing the request
        .await
        // if none off the controllers processed the request, create a default 404 Not Found response.
        .unwrap_or_else(|_| Ok(Response::<Bytes>::from_status(StatusCode::NOT_FOUND)))
        // if a controller processed the request, but resulted in an error, create a response from the error
        .unwrap_or_else(Response::<Bytes>::from_error)
        // http_tools uses a body of bytes::Bytes, but hyper expects a body of type Body, so map to a Body.
        .map(Body::from))
}

// http_tools isn't a framework, only a library, so we will need another library to create a http
// server, and send the http::Requests to our mux, as well as sending the http::Responses we generate
// to the user
#[tokio::main]
async fn main() {
    // We'll bind to 127.0.0.1:3000
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    // A `Service` is needed for every connection, so this
    // creates one from our `hello_world` function.
    let make_svc = make_service_fn(|_conn| async {
        // service_fn converts our function into a `Service`
        Ok::<_, Infallible>(service_fn(mux))
    });

    let server = Server::bind(&addr).serve(make_svc);

    // Run this server for... forever!
    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}