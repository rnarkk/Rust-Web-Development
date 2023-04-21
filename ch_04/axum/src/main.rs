use std::{
    collections::HashMap,
    fmt::Display,
    sync::Arc
};
use axum::{
    BoxError, Form, Router, Server,
    extract::{Path, Query},
    http::StatusCode,
    response::Json,
    routing::{get, post, put}
};
use serde::{Deserialize, Serialize};
use tower_http::cors::{Any, CorsLayer};
use tokio::sync::RwLock;

#[derive(Deserialize, Serialize, Debug, Clone)]
struct Question {
    id: QuestionId,
    title: String,
    content: String,
    tags: Option<Vec<String>>,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
struct QuestionId(String);

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Answer {
    id: AnswerId,
    content: String,
    question_id: QuestionId,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
struct AnswerId(String);

#[derive(Debug)]
struct Pagination {
    start: usize,
    end: usize,
}

#[derive(Clone)]
struct Store {
    questions: Arc<RwLock<HashMap<QuestionId, Question>>>,
    answers: Arc<RwLock<HashMap<AnswerId, Answer>>>,
}

impl Store {
    fn new() -> Self {
        Store {
            questions: Arc::new(RwLock::new(Self::init())),
            answers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    fn init() -> HashMap<QuestionId, Question> {
        let file = include_str!("../questions.json");
        serde_json::from_str(file).expect("can't read questions.json")
    }
}

#[derive(Debug)]
enum Error {
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

async fn handle_error(r: Rejection) -> Result<impl Reply, Rejection> {
    if let Some(error) = r.find::<Error>() {
        Ok(warp::reply::with_status(
            error.to_owned(),
            StatusCode::RANGE_NOT_SATISFIABLE,
        ))
    } else if let Some(error) = r.find::<CorsForbidden>() {
        Ok(warp::reply::with_status(
            error.to_owned(),
            StatusCode::FORBIDDEN,
        ))
    } else if let Some(error) = r.find::<BodyDeserializeError>() {
        Ok(warp::reply::with_status(
            error.to_owned(),
            StatusCode::UNPROCESSABLE_ENTITY,
        ))
    } else {
        Ok(warp::reply::with_status(
            "Route not found".to_owned(),
            StatusCode::NOT_FOUND,
        ))
    }
}

async fn get_questions(pagination: Query<Option<Pagination>>, store: Store)
    -> (StatusCode, Json<Vec<Question>>)
{
    if let Some(pagination) = pagination.0 {
        let res: Vec<Question> = store.questions.read().await.values().cloned().collect();
        let res = res[pagination.start..pagination.end];
        Ok(Json(res))
    } else {
        let res: Vec<Question> = store.questions.read().await.values().cloned().collect();
        Ok(Json(res))
    }
}

async fn add_question(
    store: Store,
    Json(question): Json<Question>,
) -> (StatusCode, String) {
    store
        .questions
        .write()
        .await
        .insert(question.id.clone(), question);

    (StatusCode::OK, "Question added".to_owned())
}

async fn update_question(
    Path(id): Path<String>,
    store: Store,
    Json(question): Json<Question>,
) -> Result<impl warp::Reply, Error> {
    match store.questions.write().await.get_mut(&QuestionId(id)) {
        Some(q) => *q = question,
        None => return Err(Error::QuestionNotFound),
    }

    Ok((StatusCode::OK, "Question updated"))
}

async fn delete_question(Path(id): Path<String>, store: Store) -> Result<> {
    match store.questions.write().await.remove(&QuestionId(id)) {
        Some(_) => return Ok((StatusCode::OK, "Question deleted")),
        None => return Err(Error::QuestionNotFound),
    }
}

#[derive(Deserialize)]
struct AddAnswer {
    content: String,
    question_id: String,
}

async fn add_answer(
    store: Store,
    Form(params): Form<AddAnswer>,
) -> Result<> {
    let answer = Answer {
        id: AnswerId("1".to_owned()),
        content: params.content,
        question_id: QuestionId(params.question_id),
    };

    store
        .answers
        .write()
        .await
        .insert(answer.id.clone(), answer);

    Ok((StatusCode::OK, "Answer added"))
}

#[tokio::main]
async fn main() {
    let store = Store::new();
    let store_filter = warp::any().map(move || store.clone());

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_headers(["content-type"])
        .allow_methods(["put", "delete", "get", "post"]);
    let app = Router::new()
        .route("/questions", get(get_questions))
        .route("/questions/:id", put(update_question).delete(delete_question).post(add_question))
        .route("/comments", post(add_answer))
        .layer(cors);

    Server::bind(&"127.0.0.1:3030".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
