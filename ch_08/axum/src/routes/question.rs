use std::collections::HashMap;

use tracing::{event, instrument, Level};
use warp::http::StatusCode;

use crate::profanity::check_profanity;
use crate::store::Store;
use crate::types::pagination::{extract_pagination, Pagination};
use crate::types::question::{NewQuestion, Question};

#[axum::debug_handler]
#[instrument]
pub async fn get_questions(
    params: Option<Query<Pagination>>,
    State(store): State<Arc<Store>>
) -> impl IntoResponse {
    event!(target: "practical_rust_book", Level::INFO, "querying questions");
    let Pagination { limit, offset } = match params {
        None => Pagination::default(),
        Some(param) => {
            event!(Level::INFO, pagination = true);
            param.0
        }
    };
    match store.get_questions(limit, offset).await {
        Ok(res) => Ok(Json(res)),
        Err(e) => Err(e),
    }
}

pub async fn update_question(
    Path(id): Path<i32>,
    State(store): State<Arc<Store>>,
    Json(question): Json<Question>
) -> impl IntoResponse {
    let title = check_profanity(question.title);
    let content = check_profanity(question.content);
    let (title, content) = tokio::join!(title, content);
    if title.is_err() {
        return Err(title.unwrap_err());
    }
    if content.is_err() {
        return Err(content.unwrap_err());
    }
    match store.update_question(question, id).await {
        Ok(res) => Ok(Json(res)),
        Err(e) => Err(e),
    }
}

pub async fn delete_question(
    Path(id): Path<i32>,
    State(store): State<Arc<Store>>
) -> impl IntoResponse {
    match store.delete_question(id).await {
        Ok(_) => Ok(format!("Question {} deleted", id)),
        Err(e) => Err(e),
    }
}

pub async fn add_question(
    State(store): State<Arc<Store>>,
    Json(NewQuestion { title, content, tags }): Json<NewQuestion>
) -> impl IntoResponse {
    let title = match check_profanity(title).await {
        Ok(res) => res,
        Err(e) => return Err(e),
    };
    let content = match check_profanity(content).await {
        Ok(res) => res,
        Err(e) => return Err(e),
    };
    match store.add_question(title, content, tags).await {
        Ok(_) => Ok("Question added"),
        Err(e) => Err(e),
    }
}
