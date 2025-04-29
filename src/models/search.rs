use crate::models::location::Location;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Search {
    pub labware_barcodes: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SearchResult {
    pub barcode: String,
    pub location: Location,
}
