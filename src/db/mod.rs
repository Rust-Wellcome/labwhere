use sqlx::sqlite::SqliteConnectOptions;
use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite};
use std::str::FromStr;
use std::{fs, time::Duration};

use crate::errors::{DatabaseError, LabwhereError};

pub mod create_db;
pub mod savable;
pub mod seeds;

/// Initiates a connection pool to the SQLite database.
///
/// # Arguments
///
/// * `url` - A string slice that holds the URL of the SQLite database.
///
/// # Returns
///
/// Returns a `Result` containing the connection pool or a `LabwhereError` if the connection fails.
///
/// # Errors
///
/// This function will return a `LabwhereError::DatabaseError` if the connection to the database fails.
///
/// # Example
///
/// ```rust
/// # #[cfg(doctest)] {
/// let pool = initiate_pool("sqlite::memory:").await.unwrap();
/// # }
/// ```
pub async fn initiate_pool(url: &str) -> Result<Pool<Sqlite>, LabwhereError> {
    let connection_options = SqliteConnectOptions::from_str(&url)
        .unwrap()
        .auto_vacuum(sqlx::sqlite::SqliteAutoVacuum::Full);
    // TODO: The number of connections and the timeout for acquiring the given number of connections needs to be taken from the config file.
    let pool_result: Result<Pool<Sqlite>, LabwhereError> = SqlitePoolOptions::new()
        .max_connections(20)
        .acquire_timeout(Duration::from_secs(5))
        .connect_with(connection_options)
        .await
        .map_err(|_| {
            LabwhereError::DatabaseError(DatabaseError {
                message: "Error creating the db pool.".to_string(),
            })
            .into()
        });
    let pool = pool_result.unwrap();
    let schemas =
        fs::read_to_string("db/schema.sql").expect("Something went wrong reading the file");
    sqlx::query(&schemas).execute(&pool).await.unwrap();
    Ok(pool)
}
