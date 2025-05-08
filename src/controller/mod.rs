use http_body::Body;
use http_body_util::combinators::BoxBody;
use http_body_util::BodyExt;
use hyper::{body::Bytes, header::CONTENT_TYPE, Error, Method, Request, Response, StatusCode};
use log::error;
use sqlx::{Pool, Sqlite};

use crate::services::empty;
use crate::services::scan::scan;
use crate::services::search::search;

pub struct Controller {}

impl Controller {

    /// Processes an incoming HTTP request and routes it to the appropriate handler.
    ///
    /// This function acts as a controller for handling HTTP requests. It checks the request's
    /// content type, validates the method and path, and routes the request to the appropriate
    /// service function (e.g., `/scan` or `/search`). If the request is invalid or the route
    /// is not found, it returns an appropriate HTTP response.
    ///
    /// # Arguments
    ///
    /// * `req` - An HTTP request implementing the `Body` trait with `Data` as `Bytes` and `Error` as `hyper::Error`.
    /// * `connection` - A reference to the SQLite connection pool used for database operations.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing an HTTP `Response` with a boxed body if successful, or a `hyper::Error`
    /// if an error occurs during processing.
    ///
    /// # Behavior
    ///
    /// - If the `Content-Type` header is not `application/json`, it returns a `400 Bad Request` response.
    /// - Routes:
    ///   - `POST /scan`: Calls the `scan` service function to process the scan request.
    ///   - `POST /search`: Calls the `search` service function to process the search request.
    ///   - Any other route: Returns a `404 Not Found` response.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The request body cannot be collected or converted into a string.
    /// - The service functions (`scan` or `search`) return an error.
    pub async fn process(
        req: Request<impl Body<Data = Bytes, Error = hyper::Error> + Send + Sync + 'static>,
        connection: &Pool<Sqlite>,
    ) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
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

        // This code fragment is a bit akin to the concept of "routes" in web frameworks.
        match (req.method(), req.uri().path()) {
            (&Method::POST, "/scan") => {
                Ok(scan(connection, &get_request_string(req).await?).await?)
            }
            (&Method::POST, "/search") => {
                Ok(search(connection, get_request_string(req).await?).await?)
            }
            _ => {
                let mut not_found = Response::new(empty());
                *not_found.status_mut() = StatusCode::NOT_FOUND;
                error!("Responding with not found");
                Ok(not_found)
            }
        }
    }
}

/// Extracts the body of an HTTP request and converts it into a `String`.
///
/// This function takes an HTTP request, collects its body asynchronously, and converts the body
/// bytes into a UTF-8 encoded string. It is useful for processing the body of incoming requests
/// in a web server.
///
/// # Arguments
///
/// * `req` - An HTTP request implementing the `Body` trait with `Data` as `Bytes` and `Error` as `hyper::Error`.
///
/// # Returns
///
/// Returns a `Result` containing the body as a `String` if successful, or a `hyper::Error` if an error occurs during
/// the collection or conversion process.
///
/// # Errors
///
/// This function will return an error if:
/// - The body cannot be collected due to an underlying I/O issue.
/// - The body bytes cannot be converted into a valid UTF-8 string.
///
/// # Examples
///
/// ```rust
/// #[cfg(doctest)] {
/// use hyper::{Request, Body};
/// use crate::controller::get_request_string;
///
/// #[tokio::main]
/// async fn main() {
///     let req = Request::builder()
///         .method("POST")
///         .uri("/example")
///         .body(Body::from("example body"))
///         .unwrap();
///
///     let result = get_request_string(req).await;
///     match result {
///         Ok(body_string) => println!("Request body: {}", body_string),
///         Err(err) => eprintln!("Error: {:?}", err),
///     }
/// }
/// # }
/// ```
async fn get_request_string(
    req: Request<impl Body<Data = Bytes, Error = hyper::Error> + Send + Sync + 'static>,
) -> Result<String, hyper::Error> {
    let boxed_body: BoxBody<Bytes, Error> = req.into_body().boxed();
    let body_bytes: Bytes = boxed_body.collect().await?.to_bytes();
    Ok(String::from_utf8(body_bytes.to_vec()).unwrap())
}
