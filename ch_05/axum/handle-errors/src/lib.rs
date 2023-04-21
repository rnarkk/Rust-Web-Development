use std::fmt::Display;
use axum::{
    BoxError,
    http::StatusCode,
    response::IntoResponse
};

#[derive(Debug)]
pub enum Error {
    ParseError(std::num::ParseIntError),
    MissingParameters,
    QuestionNotFound,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Error::ParseError(ref err) => write!(f, "Cannot parse parameter: {}", err),
            Error::MissingParameters => write!(f, "Missing parameter"),
            Error::QuestionNotFound => write!(f, "Question not found"),
        }
    }
}

impl std::error::Error for Error {}

pub async fn return_error(err: BoxError) -> impl IntoResponse {
    println!("{:?}", err);
    if err.is::<Error>() {
        (StatusCode::UNPROCESSABLE_ENTITY, err.to_string())
    // } else if let Some(error) = err.is::<CorsForbidden>() {
    //     (StatusCode::FORBIDDEN, error.to_owned())
    // } else if let Some(error) = err.is::<BodyDeserializeError>() {
    //     (StatusCode::UNPROCESSABLE_ENTITY, error.to_owned())
    } else {
        (StatusCode::NOT_FOUND, "Route not found".to_owned())
    }
}
