use std::sync::Arc;
use axum::{
    Form,
    extract::State,
    response::IntoResponse
};

use crate::{store::Store, types::answer::NewAnswer};

pub async fn add_answer(
    State(store): State<Arc<Store>>,
    Form(NewAnswer{ question_id, content }): Form<NewAnswer>,
) -> impl IntoResponse {
    match store.add_answer(question_id, content).await {
        Ok(_) => Ok("Answer added"),
        Err(e) => Err(e)
    }
}
