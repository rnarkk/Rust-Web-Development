#![warn(clippy::all)]

use axum::{
    Router, Server,
    routing::{get, post, put}
};
use tower_http::cors::{Any, CorsLayer};
use handle_errors::return_error;

mod routes;
mod store;
mod types;

use routes::{
    question::{get_questions, update_question, delete_question, add_question},
    answer::add_answer
};

#[tokio::main]
async fn main() {
    let store = store::Store::new();
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
