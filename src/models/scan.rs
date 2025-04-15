use crate::errors::LabwhereError;
use crate::models::labware::Labware;
use crate::models::location::Location;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite};

use super::labware;

#[derive(Serialize, Deserialize, Debug)]
pub struct Scan {
    pub labware_barcodes: String,
    pub location_barcode: String,
}

impl Clone for Scan {
    fn clone(&self) -> Self {
        Scan {
            labware_barcodes: self.labware_barcodes.clone(),
            location_barcode: self.location_barcode.clone(),
        }
    }
}

impl Scan {
    pub fn new(labware_barcodes: String, location_barcode: String) -> Scan {
        Scan {
            labware_barcodes,
            location_barcode,
        }
    }

    /// Creates a Scan model after validations.
    ///
    /// 1. Find the location by its barcode `location_barcode`.
    /// 2. If the location doesn't exist, it returns an error.
    /// 3. Find the labware by its barcode `labware_barcodes`.
    /// 4. If the labware exists, update its location. If it doesn't exist, create the labware in the database.
    ///
    /// # Arguments
    ///
    /// * `scan` - A `Scan` struct containing the labware and location barcodes.
    /// * `connection` - A reference to the SQLite connection pool.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the created `Scan` or a `LabwhereError` if an error occurs.
    ///
    /// # Errors
    ///
    /// This function will return a `LabwhereError` if:
    /// * The location is not found.
    /// * The labware barcode is empty.
    /// * There is a database error.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #[cfg(doctest)] {
    /// use crate::models::scan::Scan;
    /// use crate::db::initiate_pool;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let connection = initiate_pool("sqlite::memory:").await.unwrap();
    ///     let scan = Scan::new("lw-1".to_string(), "lc-1".to_string());
    ///     let result = Scan::create(scan, &connection).await;
    ///     match result {
    ///         Ok(scan) => println!("Scan created: {:?}", scan),
    ///         Err(err) => println!("Error creating scan: {:?}", err),
    ///     }
    /// }
    /// # }
    pub async fn create(scan: Scan, connection: &Pool<Sqlite>) -> Result<Scan, LabwhereError> {
        let location: Location =
            match Location::find_by_barcode(scan.location_barcode, connection).await {
                Ok(location) => location,
                Err(_) => return Err(LabwhereError::not_found_error("Location")),
            };
        let labware_barcodes = scan.labware_barcodes.clone();
        let split_barcodes = labware_barcodes.split('\n').collect::<Vec<&str>>();
        // Split the labware barcodes by newline and collect them into a vector
        // Check if the labware barcode is empty

        if split_barcodes.is_empty() {
            return Err(LabwhereError::barcode_empty_error());
        }

        for barcode in split_barcodes.iter() {
            match Labware::find_by_barcode(&barcode.to_string(), connection).await {
                Ok(mut labware) => {
                    // Scan the labware into the location
                    labware.location_id = location.id;
                    match Labware::update(&labware, connection).await {
                        Ok(lw) => lw,
                        Err(_) => return Err(LabwhereError::database_error()),
                    }
                }
                Err(error) => match error {
                    LabwhereError::BarcodeEmptyError(err) => return Err(err.into()),
                    LabwhereError::NotFoundError(_) => {
                        Labware::create(barcode.to_string(), location.id, connection)
                            .await
                            .unwrap()
                    }
                    _ => return Err(LabwhereError::database_error()), // It never reaches this point.
                },
            };
        }

        // get labware by barcode
        // if labware doesn't exist, create it
        // scan the labware into the location
        // return the scan
        Ok(Scan {
            labware_barcodes: labware_barcodes,
            location_barcode: location.barcode.unwrap(),
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::db::initiate_pool;
    use crate::errors::LabwhereError;
    use crate::models::labware::Labware;
    use crate::models::{location::Location, location_type::LocationType, scan::Scan};

    #[test]
    fn test_scan_new() {
        let scan = Scan::new("lw-bc-1".to_string(), "lc-bc-1".to_string());
        assert_eq!(scan.location_barcode, "lc-bc-1".to_string());
        assert_eq!(scan.labware_barcodes, "lw-bc-1".to_string());
    }

    #[tokio::test]
    async fn test_scan_create_no_location() {
        let scan = Scan::new("lw-bc-1".to_string(), "lc-bc-1".to_string());
        let connection = initiate_pool("sqlite::memory:").await.unwrap();
        let result = Scan::create(scan, &connection).await;
        assert!(result.is_err());
        if let Err(err) = result {
            match err {
                LabwhereError::NotFoundError(err) => {
                    assert_eq!(err.message, "Location not found!".to_string())
                }
                _ => panic!("Unexpected error"),
            }
        }
    }

    #[tokio::test]
    async fn test_scan_with_dodgy_labware_returns_an_error() {
        let connection = initiate_pool("sqlite::memory:").await.unwrap();
        let location_type = LocationType::create("Freezer".to_string(), &connection)
            .await
            .unwrap();
        let location = Location::create("location1".to_string(), location_type.id, &connection)
            .await
            .unwrap();
        let scan = Scan::new("".to_string(), location.barcode.unwrap());
        let result = Scan::create(scan, &connection).await;
        println!("{:?}", result);
        assert!(result.is_err());
        if let Err(err) = result {
            match err {
                LabwhereError::BarcodeEmptyError(err) => {
                    assert_eq!(err.message, "Barcode is empty!".to_string())
                }
                _ => panic!("Unexpected error"),
            }
        }
    }

    #[tokio::test]
    async fn test_scan_for_a_new_labware() {
        let connection = initiate_pool("sqlite::memory:").await.unwrap();
        let location_type = LocationType::create("Freezer".to_string(), &connection)
            .await
            .unwrap();
        let location = Location::create("location1".to_string(), location_type.id, &connection)
            .await
            .unwrap();

        let scan = Scan::new("lw-1".to_string(), location.barcode.clone().unwrap());
        let result: Scan = Scan::create(scan.clone(), &connection).await.unwrap();

        assert_eq!(location.barcode.unwrap(), result.location_barcode);
        assert_eq!("lw-1".to_string(), scan.labware_barcodes);
    }

    #[tokio::test]
    async fn test_scan_for_an_existing_labware() {
        let connection = initiate_pool("sqlite::memory:").await.unwrap();
        let location_type = LocationType::create("Freezer".to_string(), &connection)
            .await
            .unwrap();
        let location = Location::create("location1".to_string(), location_type.id, &connection)
            .await
            .unwrap();

        let _ = Labware::create("lw-1".to_string(), location.id, &connection)
            .await
            .unwrap();

        let scan = Scan::new("lw-1".to_string(), location.barcode.clone().unwrap());
        let result: Scan = Scan::create(scan.clone(), &connection).await.unwrap();

        assert_eq!(location.barcode.unwrap(), result.location_barcode);
        assert_eq!("lw-1".to_string(), scan.labware_barcodes);
    }

    #[tokio::test]
    async fn test_scan_for_multiple_labware_barcodes() {
        let connection = initiate_pool("sqlite::memory:").await.unwrap();
        let location_type = LocationType::create("Freezer".to_string(), &connection)
            .await
            .unwrap();
        let location = Location::create("location1".to_string(), location_type.id, &connection)
            .await
            .unwrap();

        let scan = Scan::new(
            "lw-1\nlw-2\nlw-3".to_string(),
            location.barcode.clone().unwrap(),
        );

        let result: Scan = Scan::create(scan.clone(), &connection).await.unwrap();

        let db_result: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM labwares")
            .fetch_one(&connection)
            .await
            .unwrap();

        assert_eq!(location.barcode.unwrap(), result.location_barcode);
        assert_eq!("lw-1\nlw-2\nlw-3".to_string(), scan.labware_barcodes);
        assert_eq!(3, db_result.0);
    }
}
