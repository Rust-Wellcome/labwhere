use crate::services::full;
use http_body_util::combinators::BoxBody;
use http_body_util::{BodyExt, Full};
use hyper::body::Bytes;
use hyper::header::CONTENT_TYPE;
use hyper::{Response, StatusCode};
use labwhere::models::labware::Labware;
use labwhere::models::location::Location;
use labwhere::models::search::SearchResult;
use log::error;
use sqlx::{Pool, Sqlite};

/// Searches for labware barcodes and retrieves their associated locations.
///
/// This function takes a string of labware barcodes separated by the `|` character,
/// queries the database to find the associated locations for each barcode, and
/// returns the results as a JSON response.
///
/// # Arguments
///
/// * `connection` - A reference to the database connection pool.
/// * `labware_barcodes` - A `String` containing labware barcodes separated by `|`.
///
/// # Returns
///
/// * `Ok(Response<BoxBody<Bytes, hyper::Error>>)` - A JSON response containing a list of
///   `SearchResult` objects if the operation is successful.
/// * `Err(hyper::Error)` - If an error occurs during the process.
///
/// # Errors
///
/// * Returns an internal server error response if a barcode cannot be found or
///   if there is an issue querying the database.
/// * Returns a JSON serialization error if the results cannot be serialized.
///
/// # Examples
///
/// ```rust
/// # #[cfg(doctest)] {
/// let response = search(&connection, "barcode1|barcode2".to_string()).await.unwrap();
/// assert_eq!(response.status(), StatusCode::OK);
/// # }
/// ```
pub(crate) async fn search(
    connection: &Pool<Sqlite>,
    labware_barcodes: String,
) -> std::result::Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    let mut result: Vec<SearchResult> = Vec::new();
    let split: Vec<&str> = labware_barcodes.split("\n").collect();
    for barcode in split {
        match Labware::find_by_barcode(&barcode.to_string(), connection).await {
            Ok(_) => {
                let location = Location::find_by_labware_barcode(&barcode, connection)
                    .await
                    .unwrap();
                result.push(SearchResult {
                    barcode: barcode.to_string(),
                    location,
                })
            }
            Err(err) => {
                let error_message = format!("Error: {:?}", err);
                let mut error_response = Response::new(full(error_message));
                *error_response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                error!("Error processing scan: {:?}", err);
                return Ok(error_response);
            }
        }
    }

    let json_result = serde_json::to_string(&result).unwrap();
    Ok(Response::builder()
        .header(CONTENT_TYPE, "application/json")
        .body(full(json_result))
        .unwrap())
}
