use crate::services::full;
use http_body_util::combinators::BoxBody;
use http_body_util::{BodyExt, Full};
use hyper::body::Bytes;
use hyper::header::CONTENT_TYPE;
use hyper::{Response, StatusCode};
use labwhere::errors::LabwhereError;
use labwhere::models::labware::Labware;
use labwhere::models::location::Location;
use labwhere::models::search::SearchResult;
use log::error;
use sqlx::{Pool, Sqlite};

pub(crate) async fn search(
    labware_barcodes: String,
    connection: &Pool<Sqlite>,
) -> std::result::Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    let mut result: Vec<SearchResult> = Vec::new();
    let split: Vec<&str> = labware_barcodes.split("|").collect();
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
