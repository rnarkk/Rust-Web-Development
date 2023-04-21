use axum::{
    Router, Server,
    routing::get
};

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/hello", get(|name: String| async move {
            format!("Hello, {}!", name)
        }));

    Server::bind(&"127.0.0.1:1337".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
