use axum::response::IntoResponse;

use crate::{store::Store, types::answer::NewAnswer};

pub async fn add_answer(
    State(store): State<Arc<Store>>,
    Form(AddAnswer{ question_id, content }): Form<AddAnswer>,
) -> impl IntoResponse {
    match store.add_answer(question_id, content).await {
        Ok(_) => Ok((StatusCode::OK, "Answer added")),
        Err(e) => Err(e),
    }
}
