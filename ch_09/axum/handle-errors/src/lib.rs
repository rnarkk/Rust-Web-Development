use std::fmt::Display;
use axum::{
    BoxError,
    http::StatusCode,
    response::{IntoResponse, Response}
};
use argon2::Error as ArgonError;
use reqwest::Error as ReqwestError;
use reqwest_middleware::Error as MiddlewareReqwestError;
use tracing::{event, instrument, Level};

#[derive(Debug)]
pub enum Error {
    ParseError(std::num::ParseIntError),
    MissingParameters,
    WrongPassword,
    CannotDecryptToken,
    Unauthorized,
    ArgonLibraryError(ArgonError),
    DatabaseQueryError(sqlx::Error),
    ReqwestAPIError(ReqwestError),
    MiddlewareReqwestAPIError(MiddlewareReqwestError),
    ClientError(APILayerError),
    ServerError(APILayerError),
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
            Self::ParseError(ref err) => {
                write!(f, "Cannot parse parameter: {}", err)
            }
            Self::MissingParameters => write!(f, "Missing parameter"),
            Self::WrongPassword => write!(f, "Wrong password"),
            Self::CannotDecryptToken => write!(f, "Cannot decrypt error"),
            Self::Unauthorized => write!(
                f,
                "No permission to change the underlying resource"
            ),
            Self::ArgonLibraryError(_) => {
                write!(f, "Cannot verifiy password")
            },
            Self::DatabaseQueryError(_) => {
                write!(f, "Cannot update, invalid data")
            },
            Self::ReqwestAPIError(err) => {
                write!(f, "External API error: {}", err)
            },
            Self::MiddlewareReqwestAPIError(err) => {
                write!(f, "External API error: {}", err)
            }
            Self::ClientError(err) => {
                write!(f, "External Client error: {}", err)
            }
            Self::ServerError(err) => {
                write!(f, "External Server error: {}", err)
            }
        }
    }
}

impl std::error::Error for Error {}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let res = match self {
            Self::WrongPassword => {
                event!(Level::ERROR, "Entered wrong password");
                (
                    StatusCode::UNAUTHORIZED,
                    "Wrong E-Mail/Password combination".to_string()
                )
            }
            Self::Unauthorized => {
                event!(Level::ERROR, "Not matching account id");
                (
                    StatusCode::UNAUTHORIZED,
                    "No permission to change underlying resource".to_string(),
                )
            }
            Self::DatabaseQueryError(err) => {
                event!(Level::ERROR, "Database query error");
                match err {
                    sqlx::Error::Database(err) => {
                        if err.code().unwrap().parse::<u32>().unwrap()
                            == DUPLICATE_KEY
                        {
                            (StatusCode::UNPROCESSABLE_ENTITY, "Account already exsists".to_string())
                        } else {
                            (StatusCode::UNPROCESSABLE_ENTITY, "Cannot update data".to_string())
                        }
                    }
                    _ => (StatusCode::UNPROCESSABLE_ENTITY, "Cannot update data".to_string())
                }
            }
            Self::ReqwestAPIError(err) => {
                event!(Level::ERROR, "{}", err);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error".to_owned())
            }
            Self::MiddlewareReqwestAPIError(err) => {
                event!(Level::ERROR, "{}", err);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error".to_owned())
            }
            Self::ClientError(err) => {
                event!(Level::ERROR, "{}", err);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error".to_owned())
            }
            Self::ServerError(err) => {
                event!(Level::ERROR, "{}", err);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error".to_owned())
            }
            err => {
                event!(Level::ERROR, "{}", err);
                (StatusCode::UNPROCESSABLE_ENTITY, err.to_string())
            }
        };
        res.into_response()
    }
}

const DUPLICATE_KEY: u32 = 23505;

#[instrument]
pub async fn return_error(r: BoxError) -> impl IntoResponse {
    event!(Level::WARN, "Requested route was not found");
    (StatusCode::NOT_FOUND, "Route not found".to_string())
}
