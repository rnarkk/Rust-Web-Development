use std::{
    io::{Error, ErrorKind},
    str::FromStr
};
use axum::{
    BoxError, Router, Server,
    http::StatusCode,
    response::Json,
    routing::get
};
use serde::Serialize;
use tower_http::cors::{Any, CorsLayer};

#[derive(Debug, Serialize)]
struct Question {
    id: QuestionId,
    title: String,
    content: String,
    tags: Vec<String>,
}

#[derive(Debug, Serialize)]
struct QuestionId(String);

impl Question {
    fn new(id: QuestionId, title: String, content: String, tags: Vec<String>) -> Self {
        Self { id, title, content, tags }
    }
}

impl FromStr for QuestionId {
    type Err = std::io::Error;

    fn from_str(id: &str) -> Result<Self, Self::Err> {
        match id.is_empty() {
            false => Ok(QuestionId(id.to_owned())),
            true => Err(Error::new(ErrorKind::InvalidInput, "No id provided")),
        }
    }
}

#[derive(Debug)]
struct InvalidId;

async fn get_questions() -> Result<Json<Question>, InvalidId> {
    let question = Question::new(
        QuestionId::from_str("1").expect("No id provided"),
        "First Question".to_owned(),
        "Content of question".to_owned(),
        vec!["faq".to_owned()],
    );

    match question.id.0.parse::<i32>() {
        Err(_) =>  Err(InvalidId),
        Ok(_) => Ok(Json(&question))
    }
}

async fn handle_error(err: BoxError) -> (StatusCode, String) {
    // if let Some(error) = err.is::<CorsForbidden>() {
    //     (StatusCode::FORBIDDEN, error.to_owned())
    // } else
    if err.is::<InvalidId>() {
        (StatusCode::UNPROCESSABLE_ENTITY, "No valid id presented".to_owned())
    }  else {
        (StatusCode::NOT_FOUND, "Route not found".to_owned())
    }
}

#[tokio::main]
async fn main() {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_headers(["content-type"])
        .allow_methods(["put", "delete", "get", "post"]);

    let app = Router::new()
        .route("/questions", get(get_questions))
        .layer(cors);
    // let get_items = warp::get()
    //     .and(warp::path("questions"))
    //     .and(warp::path::end())
    //     .and_then(get_questions)
    //     .recover(handle_error);

    Server::bind(&"127.0.0.1:3030".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
