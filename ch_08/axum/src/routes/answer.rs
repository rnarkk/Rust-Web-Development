use std::sync::Arc;
use axum::{
    Form,
    extract::State,
    response::IntoResponse
};

use crate::{
    profanity::check_profanity,
    store::Store,
    types::answer::NewAnswer
};

pub async fn add_answer(
    State(store): State<Arc<Store>>,
    Form(NewAnswer{ question_id, content }): Form<NewAnswer>,
) -> impl IntoResponse {
    let content = match check_profanity(content).await {
        Ok(res) => res,
        Err(e) => return Err(e),
    };
    match store.add_answer(question_id, content).await {
        Ok(_) => Ok("Answer added"),
        Err(e) => Err(e)
    }
}
