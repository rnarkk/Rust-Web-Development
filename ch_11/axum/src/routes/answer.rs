use std::collections::HashMap;
use axum::{
    http::StatusCode,
    response::IntoResponse
};

use crate::profanity::check_profanity;
use crate::store::Store;
use crate::types::{account::Session, answer::Answer};

pub async fn add_answer(
    session: Session,
    State(store): State<Arc<Store>>,
    params: HashMap<String, String>,
) -> impl IntoResponse {
    let account_id = session.account_id;
    let content = match check_profanity(params.get("content").unwrap().to_string()).await {
        Ok(res) => res,
        Err(e) => return Err(warp::reject::custom(e)),
    };

    let answer = Answer {
        content,
        question_id: params.get("question_id").unwrap().parse().unwrap(),
    };

    match store.add_answer(answer, account_id).await {
        Ok(_) => (StatusCode::OK, "Answer added"),
        Err(e) => (StatusCode::UNPROCESSABLE_ENTITY, e),
    }
}
