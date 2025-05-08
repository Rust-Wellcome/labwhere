use crate::models::location::Location;
use serde::{Deserialize, Serialize};

/// Represents a search request containing labware barcodes.
///
/// This struct is used to deserialize the JSON payload of a search request.
/// The `labware_barcodes` field contains a string of barcodes, typically separated by newlines.
///
/// # Fields
///
/// * `labware_barcodes` - A `String` containing the labware barcodes to be searched.
#[derive(Serialize, Deserialize, Debug)]
pub struct Search {
    /// A string containing the labware barcodes to be searched.
    pub labware_barcodes: String,
}

/// Represents the result of a search for labware locations.
///
/// This struct is used to serialize the JSON response of a search operation.
/// Each result contains a barcode and its associated location.
///
/// # Fields
///
/// * `barcode` - A `String` representing the labware barcode.
/// * `location` - A `Location` object representing the associated location of the labware.
#[derive(Serialize, Deserialize, Debug)]
pub struct SearchResult {
    /// A string representing the labware barcode.
    pub barcode: String,
    /// The associated location of the labware.
    pub location: Location,
}
