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
        if Self::is_post_or_put_request(&req) && !Self::is_valid_content_type(&req) {
            return Self::bad_request_response();
        }
        // This code fragment is a bit akin to the concept of "routes" in web frameworks.
        Self::route(req, connection).await
    }

    /// Checks if the HTTP request method is either POST or PUT. POST and PUT were chosen
    /// as they often consists of a request body.
    ///
    /// This function evaluates the method of the given HTTP request and returns `true`
    /// if the method is POST or PUT, otherwise it returns `false`.
    ///
    /// # Arguments
    ///
    /// * `req` - A reference to an HTTP request implementing the `Body` trait.
    ///
    /// # Returns
    ///
    /// * `bool` - `true` if the request method is POST or PUT, `false` otherwise.
    fn is_post_or_put_request(req: &Request<impl Body>) -> bool {
        req.method() == Method::POST || req.method() == Method::PUT
    }

    /// Validates if the `Content-Type` header is `application/json`.
    fn is_valid_content_type(req: &Request<impl Body<Data = Bytes, Error = hyper::Error>>) -> bool {
        req.headers()
            .get(CONTENT_TYPE)
            .map_or(false, |content_type| content_type == "application/json")
    }

    /// Returns a `400 Bad Request` response.
    fn bad_request_response() -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
        let mut response = Response::new(empty());
        *response.status_mut() = StatusCode::BAD_REQUEST;
        error!("Responding with bad request");
        Ok(response)
    }

    /// Routes an incoming HTTP request to the appropriate handler based on the method and URI.
    ///
    /// This function acts as a router for handling HTTP requests. It matches the request's method
    /// and URI path to predefined routes and calls the corresponding service function. If no route
    /// matches, it returns a `404 Not Found` response.
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
    ///
    /// # Examples
    ///
    /// ```rust
    /// #[cfg(doctest)] {
    /// use hyper::{Request, Body, Method, Response, StatusCode};
    /// use sqlx::Pool;
    /// use sqlx::Sqlite;
    /// use crate::controller::Controller;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let connection = Pool::<Sqlite>::connect("sqlite::memory:").await.unwrap();
    ///     let req = Request::builder()
    ///         .method("POST")
    ///         .uri("/scan")
    ///         .header("Content-Type", "application/json")
    ///         .body(Body::from("{\"location_barcode\": \"loc-1\", \"labware_barcodes\": \"lw-1\"}"))
    ///         .unwrap();
    ///
    ///     let response = Controller::route(req, &connection).await.unwrap();
    ///     assert_eq!(response.status(), StatusCode::OK);
    /// }
    /// # }
    /// ```
    async fn route(
        req: Request<impl Body<Data = Bytes, Error = hyper::Error> + Send + Sync + 'static>,
        connection: &Pool<Sqlite>,
    ) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
        // This code fragment is a bit akin to the concept of "routes" in web frameworks.
        match (req.method(), req.uri().path()) {
            (&Method::OPTIONS, _) => Ok(preflight().await),
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

/// Handles HTTP OPTIONS preflight requests for CORS support.
/// reference: https://users.rust-lang.org/t/hyper-http-server-how-to-send-preflight-headers-cors/68320
/// This asynchronous function constructs an HTTP response with the appropriate CORS headers
/// to allow cross-origin requests. It sets the status to 204 No Content and includes headers
/// for Access-Control-Allow-Origin, Access-Control-Allow-Headers, and Access-Control-Allow-Methods.
///
/// # Returns
///
/// Returns a Response with a boxed body and CORS headers set, suitable for responding to
/// browser preflight (OPTIONS) requests.
///
/// # Example
///
/// ```rust
/// #[cfg(doctest)] {
/// use hyper::{Response, StatusCode};
/// use hyper::body::Bytes;
/// use crate::controller::preflight;
///
///
/// let response = preflight().await;   
/// assert_eq!(response.status(), StatusCode::NO_CONTENT);
/// # }
/// ```
async fn preflight() -> Response<BoxBody<Bytes, hyper::Error>> {
    let response = Response::builder()
        .status(StatusCode::NO_CONTENT)
        .header("Access-Control-Allow-Origin", "*")
        .header("Access-Control-Allow-Headers", "*")
        .header("Access-Control-Allow-Methods", "POST, GET, OPTIONS")
        .body(empty());
    response.unwrap()
}
