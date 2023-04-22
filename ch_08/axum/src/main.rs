#![warn(clippy::all)]

use std::sync::Arc;
use axum::{
    Router, Server,
    // error_handling::HandleErrorLayer,
    routing::{get, post, put}
};
// use handle_errors::handle_error;
use http::{Method, header::CONTENT_TYPE};
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer
};
use tracing_subscriber::fmt::format::FmtSpan;

mod profanity;
mod routes;
mod store;
mod types;

use routes::{
    answer::add_answer,
    question::{get_questions, update_question, delete_question, add_question}
};
use store::Store;

#[tokio::main]
async fn main() {
    let log_filter = std::env::var("RUST_LOG").unwrap_or_else(|_| {
        "handle_errors=warn,practical_rust_book=warn,warp=warn".to_owned()
    });

    let store = Arc::new(Store::new("postgres://localhost:5432/rustwebdev").await);

    sqlx::migrate!()
        .run(&store.clone().connection)
        .await
        .expect("Cannot migrate DB");

    tracing_subscriber::fmt()
        // Use the filter we built above to determine which traces to record.
        .with_env_filter(log_filter)
        // Record an event when each span closes. This can be used to time our
        // routes' durations!
        .with_span_events(FmtSpan::CLOSE)
        .init();

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_headers([CONTENT_TYPE])
        .allow_methods([Method::PUT, Method::DELETE, Method::GET,
                        Method::POST]);
    let app = Router::new()
        .route("/questions", get(get_questions))
        .route("/questions/:id",
               put(update_question).delete(delete_question).post(add_question))
        .route("/comments", post(add_answer))
        .with_state(store)
        .layer(cors)
        .layer(TraceLayer::new_for_http());
    Server::bind(&"127.0.0.1:3030".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
