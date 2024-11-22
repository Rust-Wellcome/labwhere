use crate::errors::database_error::DatabaseError;
use sqlx::{Connection, SqliteConnection};
use std::fs;

use crate::errors::LabwhereError;

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
/// Better to use pooling instead of a single connection.
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
pub async fn init_db(url: &str) -> Result<SqliteConnection, LabwhereError> {
    match SqliteConnection::connect(url).await {
        Ok(mut conn) => {
            let schemas = fs::read_to_string("./src/db/schema.sql")
                .expect("Something went wrong reading the file");
            match sqlx::query(&schemas).execute(&mut conn).await {
                Ok(_) => Ok(conn),
                Err(_) => Err(LabwhereError::DatabaseError(DatabaseError {
                    message: "Error creating the schemas".to_string(),
                })),
            }
        }
        Err(_) => Err(LabwhereError::DatabaseError(DatabaseError {
            message: "Error connecting to the database".to_string(),
        })),
    }
}
