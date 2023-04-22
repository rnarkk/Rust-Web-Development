use std::sync::Arc;
use axum::{
    extract::{Json, Path, Query, State},
    response::IntoResponse
};
use tracing::{event, instrument, Level};

use crate::{
    store::Store,
    types::{
        pagination::Pagination,
        question::{NewQuestion, Question}
    }
};

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
    match store.add_question(title, content, tags).await {
        Ok(_) => Ok("Question added"),
        Err(e) => Err(e),
    }
}
