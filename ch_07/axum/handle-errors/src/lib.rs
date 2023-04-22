use std::fmt::Display;
use axum::{
    BoxError,
    http::StatusCode,
    response::IntoResponse
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

#[instrument]
pub async fn return_error(r: BoxError) -> impl IntoResponse {
    if let Some(crate::Error::DatabaseQueryError) = r.is() {
        event!(Level::ERROR, "Database query error");
        (StatusCode::UNPROCESSABLE_ENTITY, crate::Error::DatabaseQueryError.to_string())
    } else if let Some(error) = r.is::<CorsForbidden>() {
        event!(Level::ERROR, "CORS forbidden error: {}", error);
        (StatusCode::FORBIDDEN, error.to_string())
    } else if let Some(error) = r.is::<BodyDeserializeError>() {
        event!(Level::ERROR, "Cannot deserizalize request body: {}", error);
        (StatusCode::UNPROCESSABLE_ENTITY, error.to_string())
    } else if let Some(error) = r.is::<Error>() {
        event!(Level::ERROR, "{}", error);
        (StatusCode::UNPROCESSABLE_ENTITY, error.to_string())
    } else {
        event!(Level::WARN, "Requested route was not found");
        (StatusCode::NOT_FOUND, "Route not found".to_owned())
    }
}
