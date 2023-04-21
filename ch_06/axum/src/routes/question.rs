use std::sync::Arc;
use axum::{
    Form,
    extract::{Json, Path, State},
    http::StatusCode,
    response::IntoResponse
};
use tracing::{instrument, info};
use handle_errors::Error;
use crate::{
    store::Store,
    types::{
        pagination::Pagination,
        question::{Question, QuestionId}
    }
};

#[instrument]
pub async fn get_questions(
    State(store): State<Arc<Store>>,
    params: Option<Form<Pagination>>
) -> impl IntoResponse {
    info!("querying questions");
    if let Some(params) = params {
        info!(pagination = true);
        let res: Vec<Question> = store.questions.read().await.values().cloned().collect();
        let res = res[pagination.0.start..pagination.0.end];
        Json(res)
    } else {
        info!(pagination = false);
        let res: Vec<Question> = store.questions.read().await.values().cloned().collect();
        Json(res)
    }
}

pub async fn update_question(
    Path(id): Path<String>,
    State(store): State<Arc<Store>>,
    Json(question): Json<Question>
) -> impl IntoResponse {
    match store.questions.write().await.get_mut(&QuestionId(id)) {
        Some(q) => *q = question,
        None => return (
            StatusCode::UNPROCESSABLE_ENTITY,
            "Question not found"
        )
    }
    (StatusCode::OK, "Question updated")
}

pub async fn delete_question(
    Path(id): Path<String>,
    State(store): State<Arc<Store>>
) -> impl IntoResponse {
    match store.questions.write().await.remove(&QuestionId(id)) {
        Some(_) => (),
        None => return (
            StatusCode::UNPROCESSABLE_ENTITY,
            "Question not found"
        )
    }
    (StatusCode::OK, "Question deleted")
}

pub async fn add_question(
    State(store): State<Arc<Store>>,
    Json(question): Json<Question>
) -> impl IntoResponse {
    store
        .questions
        .write()
        .await
        .insert(question.clone().id, question);
    (StatusCode::OK, "Question added")
}
