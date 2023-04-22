use axum::extract::{Json, Query};
use tracing::{event, instrument, Level};

use crate::{
    store::Store,
    types::{
        pagination::Pagination,
        question::{NewQuestion, Question}
    }
};

#[instrument]
pub async fn get_questions(
    State(store): State<Arc<Store>>,
    params: Option<Query<Pagination>>
) -> impl IntoResponse {
    event!(target: "practical_rust_book", Level::INFO, "querying questions");
    let Pagination { limit, offset } = match params {
        None => Pagination::default(),
        Some(param) => {
            event!(Level::INFO, pagination = true);
            pagination = param.0;
        }
    };
    match store.get_questions(limit, offset).await {
        Ok(res) => Ok(Json(&res)),
        Err(e) => Err(e),
    }
}

pub async fn update_question(
    Path(id): Path<String>,
    State(store): State<Arc<Store>>,
    Json(question): Json<Question>
) -> impl IntoResponse {
    match store.update_question(question, id).await {
        Ok(res) => Ok(Json(&res)),
        Err(e) => Err(e),
    }
}

pub async fn delete_question(
    Path(id): Path<String>,
    State(store): State<Arc<Store>>
) -> impl IntoResponse {
    match store.delete_question(id).await {
        Ok(_) => Ok((
            StatusCode::OK,
            format!("Question {} deleted", id)
        )),
        Err(e) => Err(e),
    }
}

pub async fn add_question(
    State(store): State<Arc<Store>>,
    Json(question): Json<Question>
) -> impl IntoResponse {
    match store.add_question(new_question).await {
        Ok(_) => Ok((StatusCode::OK, "Question added")),
        Err(e) => Err(e),
    }
}
