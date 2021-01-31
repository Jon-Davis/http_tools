use std::{convert::Infallible, sync::Arc};
use tokio::sync::RwLock;
use std::net::SocketAddr;
use bytes::Bytes;
use http::StatusCode;
use anyhow::Result;
use hyper::{Body, Request, Response, Server};
use hyper::service::{make_service_fn, service_fn};
use http_tools::{request::{RequestExtension, HandlerResult, FilterError}, response::ResponseExtension};
use futures::executor::block_on;
use sqlx::{sqlite::SqlitePool, prelude::*};

#[macro_use] extern crate lazy_static;

// http_tools isn't a framework, only a library, so we will need another library to create a http
// server, and send the http::Requests to our mux, as well as sending the http::Responses we generate
// to the user
#[tokio::main]
async fn main() {
    // We'll bind to 127.0.0.1:3000
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    // Because our services will require a database pool, we will create a static reference to all
    // the global resources. The services object is first wrapped by a RwLock, this is incase we ever
    // want to update the config of our application while it is running, but for this example, only 
    // `read()` will be called on it. We use Arc because we want the state of a request to remain constant
    // throughout the processing, so once we get a request, we will clone the Arc, to ensure all the config
    // remains constant and thread safe. Finally we call the initialize_services function, which can be found
    // down below, and initializes our Services struct.
    lazy_static!{
        static ref SERVICES : RwLock<Arc<Services>> = RwLock::new(Arc::new(initialize_services().unwrap()));
    }
    // A `Service` is needed for every connection, so this
    // creates one from our `hello_world` function.
    let make_svc = make_service_fn(|_conn| async {
        // service_fn converts our function into a `Service`
        Ok::<_, Infallible>(service_fn(|req| async {
            // We want our config and global resources to remain constant and thread safe throughout the processing
            // of the request, but we also don't want to hold onto the RwLock for very long. So we will read
            // the lock, clone the Arc (Arc is just a reference so it is relatively* cheap to clone), then drop our
            // read lock, allowing a potential config refresher to update the config for future requests, even while we
            // are still processing this one
            let resources = SERVICES.read().await.clone();
            // Now we just call the mux and pass in the resources and our reference to the Services struct
            mux(req, resources).await
        }))
    });

    let server = Server::bind(&addr).serve(make_svc);

    // Run this server for... forever!
    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}


// The services struct will hold all of our services and configs, The mux 
// would then be responsible for calling all of the controllers in each service
struct Services {
    db_service : DbService,
    // If we had more services you would put them here
}

// This is the function we call up in the lazy_static to initialize our services struct.
// We only have one services, but if we had multiple services connecting to the same database
// we would want to share the same connection pool.
fn initialize_services() -> Result<Services> {
    // For initial setup, we will just connect to everything synchronously, but 
    // we will still make use of async libraries.
    block_on( async {
        // Normally you may want to connect to an external database, but for this example
        // we will just create one in memory. This means we will also need to initialize our tables
        let pool = SqlitePool::connect("sqlite::memory:").await?;
        let mut conn = pool.acquire().await?;
        sqlx::query("CREATE TABLE GREETINGS(USER TEXT PRIMARY KEY, GREETING TEXT)")
            .execute(&mut conn).await?;
        // Now we just need to create our Services struct. 
        Ok(Services {
            db_service: DbService {
                db: pool,
            }
        })
    })
}

// In this example, we are passing a reference to the Services struct to our mux, from there
// we are calling each controller of each service. In this example the controllers are methods of
// it's service struct, so it gets passed in a reference of all the data stored in it's struct.
async fn mux(req : Request<Body>, svc : Arc<Services>) -> Result<Response<Body>, Infallible> {
    // Run the hello_world service
    Ok(svc.db_service.hello_world_controller(&req)
        // wait for the selected service to finish processing the request
        .await
        // if none off the controllers processed the request, create a default 404 Not Found response.
        .unwrap_or_else(|_| Ok(Response::<Bytes>::from_status(StatusCode::NOT_FOUND)))
        // if a controller processed the request, but resulted in an error, create a response from the error
        .unwrap_or_else(Response::<Bytes>::from_error)
        // http_tools uses a body of bytes::Bytes, but hyper expects a body of type Body, so map to a Body.
        .map(Body::from))
}

// Normally you may want to put the DbService struct and implementation in it's own module, 
// however for this example we will just have everything in this one 
struct DbService {
    db : SqlitePool,
}

impl DbService {

    // The controller will filter the request, and if all filters pass, it will execute a handler
    // For this example the handler processes the request for relaxant information, passes that information
    // to the impl service, then takes the output of the impl service and converts it to a http::response.
    // The controller will run the hello_world service, whenever a GET /hello/{} request is received.
    async fn hello_world_controller(&self, req : &Request<Body>) -> Result<HandlerResult, FilterError> {
        req.filter_http()
            .filter_path("/hello/{}")
            .filter_method("GET")
            .async_handle(|req| async move {
                let input = req.get_path_var(1).unwrap_or("");
                let output = self.hello_world_impl(input).await?;
                Ok(Response::builder().body(Bytes::from(output))?)
            }).await
    }

    // The impl of the above service, it simply returns Hello World!
    async fn hello_world_impl(&self, name : &str) -> Result<String> {
        let mut connection = self.db.acquire().await?;
        let row = sqlx::query("SELECT * FROM GREETINGS WHERE UPPER(NAME) = UPPER($1)")
            .bind(name)
            .fetch_one(&mut connection).await;
        match row {
            Ok(row) => {
                let greeting : &str = row.try_get("GREETING")?;
                Ok(format!("{} {}!", greeting, name))
            },
            _ => Ok(format!("Hello {}!", name)),
        }
    }
}
