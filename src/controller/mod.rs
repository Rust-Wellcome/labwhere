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
    pub async fn process(
        req: Request<impl Body<Data = Bytes, Error = hyper::Error> + Send + Sync + 'static>,
        connection: &Pool<Sqlite>,
    ) -> std::result::Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
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

        // TODO: Fix repeated creation of boxed_body, boxed_bytes and string
        match (req.method(), req.uri().path()) {
            (&Method::POST, "/scan") => {
                Ok(scan(connection, &get_request_string(req).await.unwrap()).await.unwrap())
            }
            (&Method::POST, "/search") => {
                Ok(search(connection, get_request_string(req).await.unwrap(),).await.unwrap())
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

async fn get_request_string(
    req: Request<impl Body<Data = Bytes, Error = hyper::Error> + Send + Sync + 'static>,
) -> Result<String, hyper::Error> {
    let boxed_body: BoxBody<Bytes, Error> = req.into_body().boxed();
    let body_bytes: Bytes = boxed_body.collect().await?.to_bytes();
    Ok(String::from_utf8(body_bytes.to_vec()).unwrap())
}
