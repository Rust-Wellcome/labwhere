use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Search {
    pub labware_barcodes: String,
}

impl Search {
    pub fn new(labware_barcodes: String) -> Search {
        Search { labware_barcodes }
    }

    // TODO: This is where we are.
    pub fn find_all(
        labware_barcodes: String,
        connection: &Pool<Sqlite>,
    ) -> std::result::Result<Vec<SearchResult>, LabwhereError> {
        let labware_barcodes = labware_barcodes
            .split('\n')
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect::<Vec<String>>();

        let mut labwares = Vec::new();

        Ok(labwares)
    }

    // TOOD: Return search result array
    pub(crate) async fn find_locations_by_labware_barcodes(
        labware_barcodes: Vec<String>,
        connection: &Pool<Sqlite>,
    ) -> Result<Vec<Location>, LabwhereError> {
        let mut labwares: Vec<Labware> = Vec::new();
        for barcode in labware_barcodes {
            match Labware::find_by_barcode(&barcode, connection).await {
                Ok(labware) => labwares.push(labware),
                Err(_) => {
                    return Err(LabwhereError::not_found_error("Labware"));
                }
            }
        }

        let mut locations = Vec::new();
        for barcode in labware_barcodes {
            let location = Location::find_by_barcode(barcode, connection).await?;
            locations.push(location);
        }
        Ok(locations)
    }
}
// TODO: create a response struct for the search
pub struct SearchResult {
    pub barcode: String,
}
