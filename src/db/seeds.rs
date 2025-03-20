use crate::errors::LabwhereError;
use crate::models::location_type::LocationType;
use sqlx::{Pool, Sqlite};

/// Seeds the database with initial data.
///
/// # Arguments
///
/// * `connection` - A reference to the SQLite connection pool.
///
/// # Panics
///
/// This function will panic if:
/// * Seeding the location types fails.
///
/// # Examples
///
/// ```rust
/// # #[cfg(doctest)] {
/// let conn = initiate_pool("sqlite::memory:").await.unwrap();
/// seed(&conn).await;
/// # }
pub async fn seed(connection: &Pool<Sqlite>) {
    seed_location_types(connection).await.unwrap()
    // seed_location().await.unwrap()
    // seed_labware().await.unwrap()
}

/// Seeds the database with initial location types.
///
/// # Arguments
///
/// * `connection` - A reference to the SQLite connection pool.
///
/// # Returns
///
/// * `Result<(), LabwhereError>` - Returns `Ok(())` if seeding is successful, otherwise returns a `LabwhereError`.
///
/// # Errors
///
/// This function will return a `LabwhereError` if:
/// * There is an error creating any of the location types.
///
/// # Examples
///
/// ```rust
/// # #[cfg(doctest)] {
/// let conn = initiate_pool("sqlite::memory:").await.unwrap();
/// seed_location_types(&conn).await.unwrap();
/// # }
/// ```
async fn seed_location_types(connection: &Pool<Sqlite>) -> Result<(), LabwhereError> {
    LocationType::create(String::from("freezer"), connection).await?;
    LocationType::create(String::from("box"), connection).await?;

    Ok(())
}

// 1. Find location_type(s) by name
// 2. If exists, create a location(s) using the ID of the location type (i.e., Location.create() function).
// 3. Else, panic with proper logging.
async fn seed_location(connection: &Pool<Sqlite>) -> Result<(), LabwhereError> {
    // TODO: Complete this
    Ok(())
}

// 1. Find location(s) by name.
// 2. If exists, create a labware(s) using the ID of the location (i.e., Labware.create() function).
// 3. Else, panic with proper logging.
async fn seed_labware(connection: &Pool<Sqlite>) -> Result<(), LabwhereError> {
    // TODO: Complete this
    Ok(())
}
