use std::convert::Infallible;
use std::net::SocketAddr;
use tokio::net::TcpListener;

use http_body_util::Full;
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;

// Any module that is imported into here (e.g., `use abc_module;`) has its ancestry as the binary
// crate. Therefore, any function that is declared in the module (e.g., `abc_module`) under `pub(crate)`
// visibility can be accessed by the binary crate and NOT the library crate. If the module needs to be accessed
// by both crates, it needs to be made `pub`. The binary crate depends on the library crate (which has the same
// name listed in Cargo.toml); because stuff from library crate are imported in line 1 and 2.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {

    // Reference: https://hyper.rs/guides/1/server/hello-world/
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    // Create a TCPListener and bind it to local port 3000
    match TcpListener::bind(addr).await {
        Ok(listener) => {
            // Loop in and listen to incoming connections
            println!("Listening to incoming connections at port 3000");
            loop {
                let (stream, _) = listener.accept().await?;

                // TokioIo implements tokio::io traits. TokioIo is from Hyper.
                // Hyper is used here to accept and access incoming TCP packets.
                let io = TokioIo::new(stream);

                // Spawn an async Tokio task that accepts incoming connections and proxies them
                // to service functions.
                tokio::task::spawn(async move {
                    if let Err(err) = http1::Builder::new()
                        .serve_connection(io, service_fn(pong))
                        .await
                    {
                        eprintln!("Error service connection: {:?}", err);
                    }
                });
            }
        }
        Err(err) => {
            eprintln!("Error in spinning up the server: {:?}", err);
        }
    };
    Ok(())
}

/// This service function is the root-level service function. 
/// The root service function should have multiple child service functions.
/// The root-level service function delegates the processing of incoming messages based on 
/// the header contents of the incoming message.
/// Note that the incoming message is anonymised (_), but this should be taken into
/// consideration when delegating to the child services.  
/// Also, the main and child services should be in their own module hierarchies.
async fn pong(_: Request<hyper::body::Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {
    Ok(Response::new(Full::new(Bytes::from("pong!"))))
}
