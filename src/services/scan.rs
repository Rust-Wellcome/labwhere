use crate::services::empty;
use http_body_util::combinators::BoxBody;
use http_body_util::{BodyExt, Full};
use hyper::body::{Body, Bytes};
use hyper::{header::CONTENT_TYPE, Error, Method, Request, Response, Result, StatusCode};
use labwhere::models::scan::Scan;
use log::{error, info};
use sqlx::{Pool, Sqlite};
use std::pin::Pin;
use std::task::{Context, Poll};

/// Receives location barcode and labware, scans them into LabWhere.
/// - The incoming request implements `Send` trait as it is safe to be sent to another thread.
/// - The incoming request implements `Sync` trait as it is safe to be used among multiple threads.
/// This function is a service function, and is to be passed as a closure to a hyper `service_fn`
/// call.
pub async fn scan(
    req: Request<impl Body<Data = Bytes, Error = hyper::Error> + Send + Sync + 'static>,
    connection: &Pool<Sqlite>,
) -> std::result::Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    info!("Processing request for /scan endpoint");

    // Check if the content type is application/json
    match req.headers().get(CONTENT_TYPE) {
        Some(content_type) if content_type == "application/json" => {
            // Continue with the request processing
        }
        _ => {
            let mut bad_request = Response::new(empty());
            *bad_request.status_mut() = StatusCode::BAD_REQUEST;
            error!("Responding with bad request");
            return Ok(bad_request);
        }
    }

    match (req.method(), req.uri().path()) {
        (&Method::POST, "/scan") => {
            let boxed_body: BoxBody<Bytes, Error> = req.into_body().boxed();
            let body_bytes: Bytes = boxed_body.collect().await?.to_bytes();
            let string = String::from_utf8(body_bytes.to_vec()).unwrap();

            let json: Scan = match serde_json::from_str(&string) {
                Ok(scan) => scan,
                Err(_) => {
                    let mut bad_request = Response::new(empty());
                    *bad_request.status_mut() = StatusCode::BAD_REQUEST;
                    error!("Invalid JSON in request body");
                    return Ok(bad_request);
                }
            };

            match Scan::create(json, connection).await {
                Ok(scan) => {
                    let labware_count = scan.labware_barcodes.split('\n').filter(|s| !s.is_empty()).count();
                    let success_message = format!(
                        "{} labwares scanned into location {}",
                        labware_count, scan.location_barcode
                    );
                    Ok(Response::builder()
                        .header(CONTENT_TYPE, "application/json")
                        .body(full(success_message))
                        .unwrap())
                }
                Err(err) => {
                    let error_message = format!("Error: {:?}", err);
                    let mut error_response = Response::new(full(error_message));
                    *error_response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                    error!("Error processing scan: {:?}", err);
                    Ok(error_response)
                }
            }
        }
        _ => {
            let mut not_found = Response::new(empty());
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            error!("Responding with not found");
            Ok(not_found)
        }
    }
}

fn full<T: Into<Bytes>>(chunk: T) -> BoxBody<Bytes, hyper::Error> {
    Full::new(chunk.into())
        .map_err(|never| match never {})
        .boxed()
}

/// `MockBody` is a utility body written **only** for tests.
struct MockBody {
    data: &'static [u8],
}

impl MockBody {
    fn new(data: &'static [u8]) -> Self {
        Self { data }
    }
}

impl Body for MockBody {
    type Data = Bytes;
    type Error = hyper::Error;

    fn poll_frame(
        mut self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Option<Result<http_body::Frame<Bytes>>>> {
        if self.data.is_empty() {
            Poll::Ready(None)
        } else {
            let data = self.data;
            self.data = &[];
            Poll::Ready(Some(Ok(http_body::Frame::data(Bytes::from(data)))))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::services::scan::MockBody;
    use http_body_util::BodyExt;
    use hyper::{header::CONTENT_TYPE, StatusCode};
    use labwhere::db::initiate_pool;
    use labwhere::models::location::Location;
    use labwhere::models::location_type::LocationType;

    #[tokio::test]
    async fn test_scan() {
        let conn = initiate_pool("sqlite::memory:").await.unwrap();
        let location_type = LocationType::create("location-type-1".to_string(), &conn)
            .await
            .unwrap();
        let _ = Location::create("location".to_string(), location_type.id, &conn)
            .await
            .unwrap();
        let body: MockBody = MockBody::new(
            b"{
                    \"location_barcode\": \"lw-location-1\",
                    \"labware_barcodes\": \"labware1\\nlabware2\"
            }",
        );
        let req = hyper::Request::builder()
            .method("POST")
            .uri("/scan")
            .header(CONTENT_TYPE, "application/json")
            .body(body)
            .unwrap();
        let res = super::scan(req, &conn).await.unwrap();

        assert_eq!(res.status(), StatusCode::OK);

        // Convert the response body to a string
        let body_bytes = res.into_body().collect().await.unwrap().to_bytes();
        let body_string = String::from_utf8(body_bytes.to_vec()).unwrap();

        assert_eq!(
            body_string,
            "2 labwares scanned into location lw-location-1"
        );
    }

    #[tokio::test]
    async fn test_scan_without_correct_content_type() {
        let conn = initiate_pool("sqlite::memory:").await.unwrap();
        let location_type = LocationType::create("location-type-1".to_string(), &conn)
            .await
            .unwrap();
        let _ = Location::create("location".to_string(), location_type.id, &conn)
            .await
            .unwrap();
        let body: MockBody = MockBody::new(b"anything");
        let req = hyper::Request::builder()
            .method("POST")
            .uri("/scan")
            .header(CONTENT_TYPE, "text/plain")
            .body(body)
            .unwrap();
        let conn = initiate_pool("sqlite::memory:").await.unwrap();
        let res = super::scan(req, &conn).await.unwrap();
        assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    }
}
