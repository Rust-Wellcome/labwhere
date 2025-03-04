use sqlx::sqlite::SqliteConnectOptions;
use sqlx::{sqlite::SqlitePoolOptions, Connection, Error, Pool, Sqlite, SqliteConnection};
use std::str::FromStr;
use std::{fs, time::Duration};

use crate::errors::{DatabaseError, LabwhereError};

pub mod create_db;
pub mod savable;

/// Initializes a test database and injects the schemas.
///
/// The visibility of this function **cannot** be made `pub(crate)`` as the ancestry hierarchy of this module is is follows:
///     `db -> labwhere (lib)``.
/// Therefore, making the function visibility from `pub` to `pub(crate)` will make this **only** available to
/// the lib crate (which is under the same name as `labwhere`) but not to the binary crate. Because this initialization
/// logic needs to run in our binary executable as well (upon application startup), we will keep this function visibility
/// as `pub`.
///
/// Example usage:
/// ```
/// #[tokio::test]
/// async fn test_create_location_type() {
///    let mut conn = init_db("sqlite::memory:").await.unwrap();
///    let insert_query_result = sqlx::query("INSERT INTO LOCATION_TYPES (id, name) VALUES (?, ?)")
///         .bind(150_i64)
///         .bind("Freezer")
///         .execute(&mut conn)
///         .await;
///     let location_types_result =
///     sqlx::query_as::<_, LocationType>("SELECT * FROM LOCATION_TYPES")
///         .fetch_all(&mut conn)
///         .await;
///     let location_types = location_types_result.unwrap();
///     assert_eq!(location_types.len(), 1);
/// }
pub async fn init_db(url: &str) -> Result<SqliteConnection, Error> {
    let mut connection = SqliteConnection::connect(url).await?;
    let schemas =
        fs::read_to_string("./src/db/schema.sql").expect("Something went wrong reading the file");
    sqlx::query(&schemas).execute(&mut connection).await?;
    Ok(connection)
}

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
        fs::read_to_string("./src/db/schema.sql").expect("Something went wrong reading the file");
    sqlx::query(&schemas).execute(&pool).await.unwrap();
    Ok(pool)
}
