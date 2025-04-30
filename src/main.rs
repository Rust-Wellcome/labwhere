// Any module that is imported into here (e.g., `use abc_module;`) has its ancestry as the binary
// crate. Therefore, any function that is declared in the module (e.g., `abc_module`) under `pub(crate)`
// visibility can be accessed by the binary crate and NOT the library crate. If the module needs to be accessed
// by both crates, it needs to be made `pub`. The binary crate depends on the library crate (which has the same
// name listed in Cargo.toml); because stuff from library crate are imported in line 1 and 2.

use crate::controller::Controller;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use labwhere::db::create_db::create_db;
use labwhere::db::initiate_pool;
use labwhere::db::seeds::seed;
use log::{error, info, warn};
use std::env;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;

pub mod controller;
pub mod services;

/// Initiates the database by reading the configuration, creating the database, and seeding it with initial data.
///
/// # Arguments
///
/// * `config_path` - A string slice that holds the path to the configuration file.
///
/// # Returns
///
/// * `String` - The URL of the created database.
///
/// # Errors
///
/// This function will panic if:
/// * The configuration file cannot be read.
/// * The database cannot be created.
/// * The initial data cannot be inserted into the database.
///
/// # Example
///
/// ```rust
/// let url = create_database("config.yml").await;
/// println!("Database URL: {}", url);
/// ```
async fn create_database() -> String {
    match create_db(Some("db".to_string()), "labwhere").await {
        Ok(url) => {
            let conn = initiate_pool(&url).await.unwrap();

            info!("Seeding data into {}", url);

            // Seed data and allow to panic if fails.
            seed(&conn).await;

            url
        }
        Err(err) => panic!("Error in initiating the database: {:?}", err),
    }
}

// Notes
// 1. Implement graceful shutdowns : https://hyper.rs/guides/1/server/graceful-shutdown/
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Enable logging with env_logger wrapped around with Rust's log crate.
    // Set the logging level to INFO by default
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    // Read environment variable key PORT and set the value.
    // If no PORT environment varibale is set, the default is set, which is 3000.
    let port: u16 = env::var("PORT").map_or_else(
        |_| {
            warn!("Setting the default port 3000.");
            3000
        },
        |v| v.parse().unwrap(),
    );

    // Bind the server to an address
    let address = SocketAddr::from(([127, 0, 00, 1], port));

    // Create a TcpListener and bind the address to it.
    let listener = TcpListener::bind(address).await?;

    // Initiates the database by seeding it
    let url = create_database().await;
    let pool = Arc::new(initiate_pool(&url.clone()).await.unwrap());

    info!("Server running on port: {:?}", port);

    loop {
        // This loop progresses ONLY IF an incoming TCP Stream is there.
        let (stream, _) = listener.accept().await?;
        // After the loop is gone, the clone is destroyed.
        let pool_clone = Arc::clone(&pool);
        let io = TokioIo::new(stream);

        // Spawn tokio task for concurrent processing of incoming streams
        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                // This is the global service handler.
                // This service handler should delegate the request to the relevant endpoint
                .serve_connection(
                    io,
                    service_fn(|req| async {
                        // After the loop is gone, the clone is destroyed.
                        // As this task is spawn ONLY upon an incoming TCP stream, it is okay
                        // to have a connection opened.
                        
                        // The controller proxies the request to the corresponding service.
                        Controller::process(req, &pool_clone).await
                    }),
                )
                .await
            {
                error!("Error serving the connection: {:?}", err);
            }
        });
    }
}
