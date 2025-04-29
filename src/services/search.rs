use crate::services::{empty, full};
use http_body_util::combinators::BoxBody;
use http_body_util::{BodyExt, Full};
use hyper::body::Bytes;
use hyper::header::CONTENT_TYPE;
use hyper::{Response, StatusCode};
use labwhere::models::labware::Labware;
use labwhere::models::location::Location;
use labwhere::models::search::{Search, SearchResult};
use log::{error, warn};
use sqlx::{Pool, Sqlite};
use labwhere::models::scan::Scan;

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
/// let response = search(&connection, "barcode1\nbarcode2".to_string()).await.unwrap();
/// assert_eq!(response.status(), StatusCode::OK);
/// # }
/// ```
pub(crate) async fn search(
    connection: &Pool<Sqlite>,
    request_string: String,
) -> std::result::Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    let body: Search = match serde_json::from_str(&request_string) {
        Ok(search) => search,
        Err(_) => {
            let mut bad_request = Response::new(empty());
            *bad_request.status_mut() = StatusCode::BAD_REQUEST;
            error!("Invalid JSON in request body");
            return Ok(bad_request);
        }
    };
    let mut result: Vec<SearchResult> = Vec::new();
    let split: Vec<&str> = body.labware_barcodes.split("\n").collect();
    for barcode in split {
        match Labware::find_by_barcode(&barcode.to_string(), connection).await {
            Ok(_) => {
                let location = Location::find_by_labware_barcode(barcode, connection)
                    .await
                    .unwrap();
                result.push(SearchResult {
                    barcode: barcode.to_string(),
                    location,
                })
            }
            Err(err) => {
                warn!("Could not find labware barcode: {}", barcode);
            }
        }
    }

    let json_result = serde_json::to_string(&result).unwrap();
    Ok(Response::builder()
        .header(CONTENT_TYPE, "application/json")
        .body(full(json_result))
        .unwrap())
}

#[cfg(test)]
mod tests {
    use crate::services::MockBody;
    use http_body_util::combinators::BoxBody;
    use http_body_util::BodyExt;
    use hyper::body::Bytes;
    use hyper::header::CONTENT_TYPE;
    use hyper::{Error, StatusCode};
    use labwhere::db::initiate_pool;
    use labwhere::models::labware::Labware;
    use labwhere::models::location::Location;
    use labwhere::models::location_type::LocationType;
    use labwhere::models::search::{Search, SearchResult};

    #[tokio::test]
    async fn test_search() {
        let conn = initiate_pool("sqlite::memory:").await.unwrap();
        let location_type = LocationType::create("Freezer".to_string(), &conn)
            .await
            .unwrap();
        let location = Location::create("location1".to_string(), location_type.id, &conn)
            .await
            .unwrap();
        let labware1 = Labware::create("lw-1".to_string(), location.id, &conn)
            .await
            .unwrap();
        let labware2 = Labware::create("lw-2".to_string(), location.id, &conn)
            .await
            .unwrap();

        let body: MockBody = MockBody::new(
            b"{
                    \"labware_barcodes\": \"lw-1\\nlw-2\"
                }",
        );
        let req = hyper::Request::builder()
            .method("POST")
            .uri("/search")
            .header(CONTENT_TYPE, "application/json")
            .body(body)
            .unwrap();

        let boxed_body: BoxBody<Bytes, Error> = req.into_body().boxed();
        let body_bytes: Bytes = boxed_body.collect().await.unwrap().to_bytes();
        let request_string = String::from_utf8(body_bytes.to_vec()).unwrap();

        let res = super::search(&conn, request_string)
            .await
            .unwrap();

        assert_eq!(res.status(), StatusCode::OK);

        // Convert the response body to a string
        let response_body_bytes = res.into_body().collect().await.unwrap().to_bytes();
        let response_body_string = String::from_utf8(response_body_bytes.to_vec()).unwrap();

        assert_eq!(response_body_string, "[{\"barcode\":\"lw-1\",\"location\":{\"id\":1,\"name\":\"location1\",\"barcode\":\"lw-location1-1\",\"location_type_id\":1}},{\"barcode\":\"lw-2\",\"location\":{\"id\":1,\"name\":\"location1\",\"barcode\":\"lw-location1-1\",\"location_type_id\":1}}]");
    }
}
