use labwhere::db::init_db;
use labwhere::errors::database_error::ConnectivityError;
use labwhere::errors::not_found_error::NotFoundError;
use labwhere::errors::LabwhereError;
use labwhere::models::location_type::LocationType;

// Any module that is imported into here (e.g., `use abc_module;`) has its ancestry as the binary
// crate. Therefore, any function that is declared in the module (e.g., `abc_module`) under `pub(crate)`
// visibility can be accessed by the binary crate and NOT the library crate. If the module needs to be accessed
// by both crates, it needs to be made `pub`. The binary crate depends on the library crate (which has the same
// name listed in Cargo.toml); because stuff from library crate are imported in line 1 and 2.

#[tokio::main]
async fn main() -> Result<(), LabwhereError> {
    // Another option is to use SqlitePool. A pool gives a bunch of active connections and will
    //  resolve a connection from the pool when an database operation starts.
    let mut conn = init_db("sqlite::memory:").await.unwrap();

    match sqlx::query("INSERT INTO location_types (id, name) VALUES (?, ?)")
        .bind(150_i64)
        .bind("Freezer")
        .execute(&mut conn)
        .await
    {
        Ok(_) => {
            let result: Vec<LocationType> =
                match sqlx::query_as::<_, LocationType>("SELECT * FROM location_types")
                    .fetch_all(&mut conn)
                    .await
                {
                    Ok(result) => result,
                    Err(_) => {
                        return Err(LabwhereError::NotFound(NotFoundError {
                            message: "Not found".to_string(),
                        }));
                    }
                };
            assert_eq!(result.len(), 1);
        }
        Err(_) => {
            return Err(LabwhereError::ConnectivityError(ConnectivityError {
                message: "Not found".to_string(),
            }))
        }
    };
    Ok(())
}
