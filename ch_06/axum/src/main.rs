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
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer
};
use tracing_subscriber::fmt::format::FmtSpan;

use routes::{
    answer::add_answer,
    question::{get_questions, update_question, delete_question, add_question}
};
use store::Store;

#[tokio::main]
async fn main() {
    let log_filter = std::env::var("RUST_LOG").unwrap_or_else(|_| "practical_rust_book=info,warp=error".to_owned());

    let store = Arc::new(Store::new());

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
        .allow_methods([Method::PUT, Method::DELETE, Method::GET, Method::POST]);

    // let get_questions = warp::get()
    //     .and(warp::path("questions"))
    //     .and(warp::path::end())
    //     .and(warp::query())
    //     .and(store_filter.clone())
    //     .and_then(routes::question::get_questions)
    //     .with(warp::trace(|info| {
    //         tracing::info_span!(
    //             "get_questions request",
    //             method = %info.method(),
    //             path = %info.path(),
    //             id = %uuid::Uuid::new_v4(),
    //         )})
    //     );

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
