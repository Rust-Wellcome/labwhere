// Any module that is imported into here (e.g., `use abc_module;`) has its ancestry as the binary
// crate. Therefore, any function that is declared in the module (e.g., `abc_module`) under `pub(crate)`
// visibility can be accessed by the binary crate and NOT the library crate. If the module needs to be accessed
// by both crates, it needs to be made `pub`. The binary crate depends on the library crate (which has the same
// name listed in Cargo.toml); because stuff from library crate are imported in line 1 and 2.

use crate::config::{read_config, AppConfig};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use labwhere::db::create_db::create_db;
use labwhere::db::init_db;
use labwhere::errors::LabwhereError;
use labwhere::models::location::Location;
use labwhere::models::location_type::LocationType;
use log::{error, info, warn};
use sqlx::{Error, SqliteConnection};
use std::env;
use std::future::Future;
use std::net::SocketAddr;
use tokio::fs;
use tokio::net::TcpListener;

pub mod config;
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
/// let url = initiate_database("config.yml").await;
/// println!("Database URL: {}", url);
/// ```
async fn initiate_database(config_path: &str) -> String {
    info!("Config location: {}", config_path);
    let config: AppConfig = read_config(config_path).await.unwrap();

    match create_db(
        config.database_directory,
        &config.environment.unwrap().to_string(),
    )
    .await
    {
        Ok(url) => {
            let mut conn = init_db(&url).await.unwrap();

            info!("Seeding data into {}", url);
            let location_type = LocationType::create("location-type-1".to_string(), &mut conn)
                .await
                .unwrap();
            let _ = Location::create("location".to_string(), location_type.id, &mut conn)
                .await
                .unwrap();

            url
        }
        Err(_) => panic!("Error in initiating the database."),
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

    // Reads config
    let config_path: String =
        env::var("CONFIG_PATH").unwrap_or_else(|_| "./config.yml".to_string());

    // Initiates the database by seeding it
    let url = initiate_database(&config_path).await;

    info!("Server running on port: {:?}", port);

    loop {
        // This loop progresses ONLY IF an incoming TCP Stream is there.
        let (stream, _) = listener.accept().await?;
        // After the loop is gone, the clone is destroyed.
        let url_clone = url.clone();
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
                        //
                        // This is similar to having a database connection open for each client.
                        let mut connection = init_db(&url_clone.clone()).await.unwrap();
                        services::scan::scan(req, &mut connection).await
                    }),
                )
                .await
            {
                error!("Error serving the connection: {:?}", err);
            }
        });
    }
}
