use std::fmt::Display;
use axum::{
    BoxError,
    http::StatusCode,
    response::{IntoResponse, Response}
};
use tracing::{event, Level, instrument};

#[derive(Debug)]
pub enum Error {
    ParseError(std::num::ParseIntError),
    MissingParameters,
    DatabaseQueryError,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match &*self {
            Error::ParseError(ref err) => write!(f, "Cannot parse parameter: {}", err),
            Error::MissingParameters => write!(f, "Missing parameter"),
            Error::DatabaseQueryError => write!(f, "Cannot update, invalid data."),
        }
    }
}

impl std::error::Error for Error {}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let body = format!("{}", self);
        (StatusCode::UNPROCESSABLE_ENTITY, body).into_response()
    }
}

#[instrument]
pub async fn handle_error(err: BoxError) -> impl IntoResponse {
    if err.is::<Error>() {
        match err.as_ref().downcast_ref::<Error>().unwrap() {
            Error::DatabaseQueryError => {
                event!(Level::ERROR, "Database query error");
                (StatusCode::UNPROCESSABLE_ENTITY, Error::DatabaseQueryError.to_string())
            }
            err => {
                event!(Level::ERROR, "{}", err);
                (StatusCode::UNPROCESSABLE_ENTITY, err.to_string())
            }
        }
    } else {
        event!(Level::WARN, "Requested route was not found");
        (StatusCode::NOT_FOUND, "Route not found".to_owned())
    }
}
