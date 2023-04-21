#![warn(clippy::all)]

pub mod config;
mod profanity;
mod routes;
mod store;
pub mod types;

pub use handle_errors;

use std::{
    net::SocketAddr,
    sync::Arc
};
use axum::{http::Method, Filter, Reply};
use tokio::sync::{oneshot, oneshot::Sender};
use tracing_subscriber::fmt::format::FmtSpan;

use config::Config;
use routes::{
    authentication::{login, register},
    question::{get_questions, update_question, delete_question, add_question},
    answer::add_answer
};
use store::Store;

pub struct OneshotHandler {
    pub sender: Sender<i32>,
}

async fn build_routes(store: Arc<Store>) -> impl Filter<Extract = impl Reply> + Clone {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_headers(["content-type"])
        .allow_methods(["put", "delete", "get", "post"]);;

    let app = Router::new()
        .route("/questions", get(get_questions))
        .route("/questions/:id", put(update_question).delete(delete_question).post(add_question))
        .route("/comments", post(add_answer))
        .layer(cors)
        .with_state(state);

    let update_question = warp::put()
        .and(warp::path("questions"))
        .and(warp::path::param::<i32>())
        .and(warp::path::end())
        .and(routes::authentication::auth())
        .and(store_filter.clone())
        .and(warp::body::json())
        .and_then(routes::question::update_question);

    let delete_question = warp::delete()
        .and(warp::path("questions"))
        .and(warp::path::param::<i32>())
        .and(warp::path::end())
        .and(routes::authentication::auth())
        .and(store_filter.clone())
        .and_then(routes::question::delete_question);

    let add_question = warp::post()
        .and(warp::path("questions"))
        .and(warp::path::end())
        .and(routes::authentication::auth())
        .and(store_filter.clone())
        .and(warp::body::json())
        .and_then(routes::question::add_question);

    let add_answer = warp::post()
        .and(warp::path("answers"))
        .and(warp::path::end())
        .and(routes::authentication::auth())
        .and(store_filter.clone())
        .and(warp::body::form())
        .and_then(routes::answer::add_answer);

    let registration = warp::post()
        .and(warp::path("registration"))
        .and(warp::path::end())
        .and(store_filter.clone())
        .and(warp::body::json())
        .and_then(routes::authentication::register);

    let login = warp::post()
        .and(warp::path("login"))
        .and(warp::path::end())
        .and(store_filter.clone())
        .and(warp::body::json())
        .and_then(routes::authentication::login);

    get_questions
        .or(update_question)
        .or(add_question)
        .or(delete_question)
        .or(add_answer)
        .or(registration)
        .or(login)
        .with(cors)
        .with(warp::trace::request())
        .recover(handle_errors::return_error)
}

pub async fn setup_store(config: &Config)
    -> Result<Store, handle_errors::Error>
{
    let store = Store::new(&format!(
        "postgres://{}:{}@{}:{}/{}",
        config.db_user, config.db_password, config.db_host, config.db_port, config.db_name
    ))
    .await
    .map_err(handle_errors::Error::DatabaseQueryError)?;

    sqlx::migrate!()
        .run(&store.clone().connection)
        .await
        .map_err(handle_errors::Error::MigrationError)?;

    let log_filter = format!(
        "handle_errors={},rust_web_dev={},warp={}",
        config.log_level, config.log_level, config.log_level
    );

    tracing_subscriber::fmt()
        // Use the filter we built above to determine which traces to record.
        .with_env_filter(log_filter)
        // Record an event when each span closes. This can be used to time our
        // routes' durations!
        .with_span_events(FmtSpan::CLOSE)
        .init();

    Ok(store)
}

pub async fn run(config: Config, store: Store) {
    let routes = build_routes(store).await;
    Server::bind(&"0.0.0.1:{config.port}".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

pub async fn oneshot(store: Store) -> OneshotHandler {
    let routes = build_routes(store).await;
    let (tx, rx) = oneshot::channel::<i32>();

    let socket: SocketAddr = "127.0.0.1:3030"
        .to_string()
        .parse()
        .expect("Not a valid address");

    let (_, server) = warp::serve(routes).bind_with_graceful_shutdown(socket, async {
        rx.await.ok();
    });

    tokio::task::spawn(server);

    OneshotHandler { sender: tx }
}
