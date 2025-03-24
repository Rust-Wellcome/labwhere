use crate::errors::LabwhereError;
use crate::models::labware::Labware;
use crate::models::location::Location;
use crate::models::location_type::LocationType;
use crate::models::scan::Scan;
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
    seed_location_types(connection).await.unwrap();
    seed_location(connection).await.unwrap();
    seed_labware(connection).await.unwrap();
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
    match LocationType::create(String::from("freezer"), connection).await {
        Ok(_) => (),
        Err(err) => match err {
            LabwhereError::UniqueConstraintViolation(_) => (),
            _ => panic!("Error creating location type"),
        },
    }
    match LocationType::create(String::from("box"), connection).await {
        Ok(_) => (),
        Err(err) => match err {
            LabwhereError::UniqueConstraintViolation(_) => (),
            _ => panic!("Error creating location type"),
        },
    }
    Ok(())
}

// 1. Find location_type(s) by name
// 2. If exists, create a location(s) using the ID of the location type (i.e., Location.create() function).
// 3. Else, panic with proper logging.
async fn seed_location(connection: &Pool<Sqlite>) -> Result<(), LabwhereError> {
    let freezer_location_type =
        LocationType::find_by_name(String::from("freezer"), connection).await?;
    let box_location_type = LocationType::find_by_name(String::from("box"), connection).await?;
    match Location::create(
        "Freezer 1".to_string(),
        freezer_location_type.id,
        connection,
    )
    .await
    {
        Ok(_) => (),
        Err(err) => match err {
            LabwhereError::UniqueConstraintViolation(_) => (),
            _ => panic!("Error creating location type"),
        },
    }
    match Location::create("Box 1".to_string(), box_location_type.id, connection).await {
        Ok(_) => (),
        Err(err) => match err {
            LabwhereError::UniqueConstraintViolation(_) => (),
            _ => panic!("Error creating location type"),
        },
    }
    Ok(())
}

// 1. Find location(s) by name.
// 2. If exists, create a labware(s) using the ID of the location (i.e., Labware.create() function).
// 3. Else, panic with proper logging.
async fn seed_labware(connection: &Pool<Sqlite>) -> Result<(), LabwhereError> {
    let freezer_location = Location::find_by_name("Freezer 1".to_string(), connection).await?;
    let box_location = Location::find_by_name("Box 1".to_string(), connection).await?;

    Scan::create(
        Scan {
            labware_barcode: "labware-1".to_string(),
            location_barcode: freezer_location.barcode.clone().unwrap().to_string(),
        },
        connection,
    )
    .await?;

    Scan::create(
        Scan {
            labware_barcode: "labware-2".to_string(),
            location_barcode: freezer_location.barcode.unwrap().to_string(),
        },
        connection,
    )
    .await?;

    Scan::create(
        Scan {
            labware_barcode: "labware-3".to_string(),
            location_barcode: box_location.barcode.clone().unwrap().to_string(),
        },
        connection,
    )
    .await?;

    Scan::create(
        Scan {
            labware_barcode: "labware-4".to_string(),
            location_barcode: box_location.barcode.unwrap().to_string(),
        },
        connection,
    )
    .await?;

    Ok(())
}
