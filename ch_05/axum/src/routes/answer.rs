use std::{
    collections::HashMap,
    sync::Arc
};
use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse
};

use crate::{
    store::Store,
    types::{
        answer::{Answer, AnswerId},
        question::QuestionId,
    }
};

pub async fn add_answer(
    params: HashMap<String, String>,
    State(store): State<Arc<Store>>
) -> impl IntoResponse {
    let answer = Answer {
        id: AnswerId("1".to_string()),
        content: params.get("content").unwrap().to_string(),
        question_id: QuestionId(params.get("questionId").unwrap().to_string()),
    };
    store.answers.write().await.insert(answer.id.clone(), answer);
    Ok(warp::reply::with_status("Answer added", StatusCode::OK))
}
