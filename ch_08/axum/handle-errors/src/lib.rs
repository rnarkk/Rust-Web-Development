use std::fmt::Display;
use axum::{
    BoxError,
    http::StatusCode,
    response::{IntoResponse, Response}
};
use tracing::{event, Level, instrument};
use reqwest::Error as ReqwestError;
use reqwest_middleware::Error as MiddlewareReqwestError;


#[derive(Debug)]
pub enum Error {
    ParseError(std::num::ParseIntError),
    MissingParameters,
    DatabaseQueryError,
    ReqwestAPIError(ReqwestError),
    MiddlewareReqwestAPIError(MiddlewareReqwestError),
    ClientError(APILayerError),
    ServerError(APILayerError)
}

#[derive(Debug, Clone)]
pub struct APILayerError {
    pub status: u16,
    pub message: String,
}

impl Display for APILayerError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Status: {}, Message: {}", self.status, self.message)
    }
}

impl std::error::Error for APILayerError {}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match &*self {
            Self::ParseError(ref err) => write!(f, "Cannot parse parameter: {}", err),
            Self::MissingParameters => write!(f, "Missing parameter"),
            Self::DatabaseQueryError => write!(f, "Cannot update, invalid data."),
            Self::ReqwestAPIError(err) => write!(f, "External API error: {}", err),
            Self::MiddlewareReqwestAPIError(err) => write!(f, "External API error: {}", err),
            Self::ClientError(err) => write!(f, "External Client error: {}", err),
            Self::ServerError(err) => write!(f, "External Server error: {}", err),
        }
    }
}

impl std::error::Error for Error {}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        match &*self {
            Self::DatabaseQueryError => {
                event!(Level::ERROR, "Database query error");
                (StatusCode::UNPROCESSABLE_ENTITY, self.to_string())
            }
            Self::ReqwestAPIError(err) => {
                event!(Level::ERROR, "{}", err);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error".to_string())
            }
            Self::MiddlewareReqwestAPIError(err) => {
                event!(Level::ERROR, "{}", err);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error".to_string())
            }
            Self::ClientError(err) => {
                event!(Level::ERROR, "{}", err);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error".to_string())
            }
            Self::ServerError(err) => {
                event!(Level::ERROR, "{}", err);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error".to_string())
            }
            err => {
                event!(Level::ERROR, "{}", err);
                (StatusCode::UNPROCESSABLE_ENTITY, error.to_string())
            }
        }
    }
}


#[instrument]
pub async fn handle_error(err: BoxError) -> impl IntoResponse {
    event!(Level::WARN, "Requested route was not found");
    Ok(warp::reply::with_status(
        "Route not found".to_string(),
        StatusCode::NOT_FOUND,
    ))
}
