use log::__private_api::loc;
use crate::errors::LabwhereError;
use crate::models::labware::Labware;
use crate::models::location::Location;
use serde::{Deserialize, Serialize};
use sqlx::sqlite::SqliteConnection;

#[derive(Serialize, Deserialize, Debug)]
pub struct Scan {
    labware_barcode: String,
    location_barcode: String,
}

impl Clone for Scan {
    fn clone(&self) -> Self {
        Scan {
            labware_barcode: self.labware_barcode.clone(),
            location_barcode: self.location_barcode.clone()
        }
    }
}

impl Scan {
    pub fn new(labware_barcode: String, location_barcode: String) -> Scan {
        Scan {
            labware_barcode,
            location_barcode,
        }
    }

    /// Creates a Scan model after validations.
    ///
    /// 1. Find the location by its barcode `location_barcode`.
    /// 2. If the location doesn't exist, it would err. The service would respond to the client
    /// depending on the error type.
    /// 3. Find the labware by its barcode `labware_barcode`.
    /// 4. If the labware exists, return it. If it doesn't exist, create the labware in the database.
    pub async fn create(
        scan: Scan,
        connection: &mut SqliteConnection,
    ) -> Result<Scan, LabwhereError> {
        // TODO: Complete this function.

        // let location_result = Location::find_by_barcode(scan.location_barcode);
        let location: Location =
            match Location::find_by_barcode(scan.location_barcode, connection).await {
                Ok(location) => location,
                Err(_) => return Err(LabwhereError::not_found_error("Location")),
            };

        let labware = match Labware::find_by_barcode(&scan.labware_barcode, connection).await {
            Ok(mut labware) => {
                // Scan the labware into the location
                labware.location_id = location.id;
                match Labware::update(&labware, connection).await {
                    Ok(lw) => lw,
                    Err(_) => return Err(LabwhereError::database_error())
                }
            },
            Err(error) => match error {
                LabwhereError::BarcodeEmptyError(err) => return Err(err.into()),
                LabwhereError::NotFoundError(_) => {
                    Labware::create(scan.labware_barcode, location.id, connection)
                        .await
                        .unwrap()
                }
                _ => return Err(LabwhereError::database_error()), // It never reaches this point.
            },
        };

        // get labware by barcode
        // if labware doesn't exist, create it
        // scan the labware into the location
        // return the scan

        Ok(Scan {
            labware_barcode: labware.barcode.to_string(),
            location_barcode: location.barcode.unwrap(),
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::db::init_db;
    use crate::errors::LabwhereError;
    use crate::models::{location::Location, location_type::LocationType, scan::Scan};
    use crate::models::labware::Labware;

    #[test]
    fn test_scan_new() {
        let scan = Scan::new("lw-bc-1".to_string(), "lc-bc-1".to_string());
        assert_eq!(scan.location_barcode, "lc-bc-1".to_string());
        assert_eq!(scan.labware_barcode, "lw-bc-1".to_string());
    }

    #[tokio::test]
    async fn test_scan_create_no_location() {
        let scan = Scan::new("lw-bc-1".to_string(), "lc-bc-1".to_string());
        let mut connection = init_db("sqlite::memory:").await.unwrap();
        let result = Scan::create(scan, &mut connection).await;
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
        let mut connection = init_db("sqlite::memory:").await.unwrap();
        let location_type = LocationType::create("Freezer".to_string(), &mut connection)
            .await
            .unwrap();
        let location = Location::create("location1".to_string(), location_type.id, &mut connection)
            .await
            .unwrap();
        let scan = Scan::new("".to_string(), location.barcode.unwrap());
        let result = Scan::create(scan, &mut connection).await;
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
        let mut connection = init_db("sqlite::memory:").await.unwrap();
        let location_type = LocationType::create("Freezer".to_string(), &mut connection)
            .await
            .unwrap();
        let location = Location::create("location1".to_string(), location_type.id, &mut connection)
            .await
            .unwrap();

        let scan = Scan::new("lw-1".to_string(), location.barcode.clone().unwrap());
        let result: Scan = Scan::create(scan.clone(), &mut connection).await.unwrap();

        assert_eq!(location.barcode.unwrap(), result.location_barcode);
        assert_eq!("lw-1".to_string(), scan.labware_barcode);
    }

    #[tokio::test]
    async fn test_scan_for_an_existing_labware() {
        let mut connection = init_db("sqlite::memory:").await.unwrap();
        let location_type = LocationType::create("Freezer".to_string(), &mut connection)
            .await
            .unwrap();
        let location = Location::create("location1".to_string(), location_type.id, &mut connection)
            .await
            .unwrap();

        let labware = Labware::create("lw-1".to_string(), location.id, &mut connection)
            .await
            .unwrap();

        let scan = Scan::new("lw-1".to_string(), location.barcode.clone().unwrap());
        let result: Scan = Scan::create(scan.clone(), &mut connection).await.unwrap();

        assert_eq!(location.barcode.unwrap(), result.location_barcode);
        assert_eq!("lw-1".to_string(), scan.labware_barcode);
    }
}
