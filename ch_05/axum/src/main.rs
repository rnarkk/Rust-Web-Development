#![warn(clippy::all)]

mod routes;
mod store;
mod types;

use std::sync::Arc;
use axum::{
    Router, Server,
    routing::{get, post, put}
};
use http::{Method, header::CONTENT_TYPE};
use tower_http::cors::{Any, CorsLayer};

use routes::{
    answer::add_answer,
    question::{get_questions, update_question, delete_question, add_question}
};
use store::Store;

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
