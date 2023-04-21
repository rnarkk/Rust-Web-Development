use std::sync::Arc;
use axum::{
    extract::{Form, State},
    http::StatusCode,
    response::IntoResponse
};
use serde::{Deserialize, Serialize};
use crate::{
    store::Store,
    types::{
        answer::{Answer, AnswerId},
        question::QuestionId,
    }
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AddAnswer {
    pub content: String,
    pub question_id: String,
}

pub async fn add_answer(
    State(store): State<Arc<Store>>,
    Form(params): Form<AddAnswer>,
) -> impl IntoResponse {
    let answer = Answer {
        id: AnswerId("1".to_string()),
        content: params.content,
        question_id: QuestionId(params.question_id.into()),
    };
    store.answers.write().await.insert(answer.id.clone(), answer);
    (StatusCode::OK, "Answer added")
}
