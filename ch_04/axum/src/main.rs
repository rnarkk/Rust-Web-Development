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
use http::{Method, header::CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use tower_http::cors::{Any, CorsLayer};
use tokio::sync::RwLock;

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Question {
    id: QuestionId,
    title: String,
    content: String,
    tags: Option<Vec<String>>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, Hash)]
struct QuestionId(String);

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Answer {
    id: AnswerId,
    content: String,
    question_id: QuestionId,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, Hash)]
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

async fn handle_error(r: Rejection) -> impl IntoResponse {
    if let Some(error) = r.find::<Error>() {
        (StatusCode::RANGE_NOT_SATISFIABLE, error.to_owned())
    } else if let Some(error) = r.find::<CorsForbidden>() {
        (StatusCode::FORBIDDEN, error.to_owned())
    } else if let Some(error) = r.find::<BodyDeserializeError>() {
        (StatusCode::UNPROCESSABLE_ENTITY, error.to_owned())
    } else {
        (StatusCode::NOT_FOUND, "Route not found".to_owned())
    }
}

async fn get_questions(
    pagination: Option<Query<Pagination>>,
    State(store): State<Arc<Store>>,
) -> impl IntoResponse {
    if let Some(pagination) = pagination.0 {
        let res: Vec<Question> = store.questions.read().await.values().cloned().collect();
        let res = res[pagination.start..pagination.end];
        Json(res)
    } else {
        let res: Vec<Question> = store.questions.read().await.values().cloned().collect();
        Json(res)
    }
}

async fn add_question(
    State(store): State<Arc<Store>>,
    Json(question): Json<Question>,
) -> impl IntoResponse {
    store.questions.write().await.insert(question.id.clone(), question);
    (StatusCode::OK, "Question added".to_owned())
}

async fn update_question(
    Path(id): Path<String>,
    State(store): State<Arc<Store>>,
    Json(question): Json<Question>,
) -> impl IntoResponse {
    match store.questions.write().await.get_mut(&QuestionId(id)) {
        Some(q) => *q = question,
        None => return (
            Error::QuestionNotFound
        )
    }
    (StatusCode::OK, "Question updated")
}

async fn delete_question(
    Path(id): Path<String>,
    State(store): State<Arc<Store>>
) -> impl IntoResponse {
    match store.questions.write().await.remove(&QuestionId(id)) {
        Some(_) => (StatusCode::OK, "Question deleted"),
        None => (Error::QuestionNotFound),
    }
}

#[derive(Deserialize)]
struct AddAnswer {
    content: String,
    question_id: String,
}

async fn add_answer(
    State(store): State<Arc<Store>>,
    Form(params): Form<AddAnswer>,
) -> impl IntoResponse {
    let answer = Answer {
        id: AnswerId("1".to_owned()),
        content: params.content,
        question_id: QuestionId(params.question_id),
    };
    store.answers.write().await.insert(answer.id.clone(), answer);
   (StatusCode::OK, "Answer added")
}

#[tokio::main]
async fn main() {
    let store = Arc::new(Store::new());
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_headers([CONTENT_TYPE])
        .allow_methods([Method::PUT, Method::DELETE, Method::GET, Method::POST]);
    let app = Router::new()
        .route("/questions", get(get_questions))
        .route("/questions/:id",
               put(update_question).delete(delete_question).post(add_question))
        .route("/comments", post(add_answer))
        .with_state(store)
        .layer(cors);
    Server::bind(&"127.0.0.1:3030".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
