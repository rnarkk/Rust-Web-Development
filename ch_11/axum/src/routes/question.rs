use std::{
    collections::HashMap,
    sync::Arc
};
use axum::{
    extract::Path,
    http::StatusCode,
    response::{IntoResponse, Json}
};
use tracing::{event, instrument, Level};

use crate::profanity::check_profanity;
use crate::store::Store;
use crate::types::account::Session;
use crate::types::pagination::{extract_pagination, Pagination};
use crate::types::question::{NewQuestion, Question};

#[instrument]
pub async fn get_questions(
    Query(pagination): Query<Option<Pagination>>,
    State(store): State<Arc<Store>>,
) -> Result<Json<Vec<Question>>> {
    event!(target: "practical_rust_book", Level::INFO, "querying questions");
    let Pagination { limit, offset } = match pagination.0 {
        Some(pagination) => {
            event!(Level::INFO, pagination = true);
            pagination
        }
        None => Pagination::default()
    };

    match store.get_questions(limit, offset).await {
        Ok(res) => Ok(Json(&res)),
        Err(e) => Err(e),
    }
}

pub async fn update_question(
    Path(id): Path<i32>,
    session: Session,
    State(store): State<Arc<Store>>,
    Json(question): Json<Question>,
) -> impl IntoResponse {
    let account_id = session.account_id;
    if store.is_question_owner(id, &account_id).await? {
        let title = check_profanity(question.title);
        let content = check_profanity(question.content);

        let (title, content) = tokio::join!(title, content);

        if title.is_ok() && content.is_ok() {
            let question = Question {
                id: question.id,
                title: title.unwrap(),
                content: content.unwrap(),
                tags: question.tags,
            };
            match store.update_question(question, id, account_id).await {
                Ok(res) => Ok(warp::reply::json(&res)),
                Err(e) => Err(warp::reject::custom(e)),
            }
        } else {
            Err(warp::reject::custom(
                title.expect_err("Expected API call to have failed here"),
            ))
        }
    } else {
        Err(warp::reject::custom(handle_errors::Error::Unauthorized))
    }
}

pub async fn delete_question(
    Path(id): Path<i32>,
    session: Session,
    State(store): State<Arc<Store>>,
) -> impl IntoResponse {
    let account_id = session.account_id;
    if store.is_question_owner(id, &account_id).await? {
        match store.delete_question(id, account_id).await {
            Ok(_) => Ok(warp::reply::with_status(
                format!("Question {} deleted", id),
                StatusCode::OK,
            )),
            Err(e) => Err(warp::reject::custom(e)),
        }
    } else {
        Err(warp::reject::custom(handle_errors::Error::Unauthorized))
    }
}

pub async fn add_question(
    session: Session,
    State(store): State<Arc<Store>>,
    Json(new_question): Json<NewQuestion>,
) -> impl IntoResponse {
    let account_id = session.account_id;
    let title = match check_profanity(new_question.title).await {
        Ok(res) => res,
        Err(e) => return Err(warp::reject::custom(e)),
    };

    let content = match check_profanity(new_question.content).await {
        Ok(res) => res,
        Err(e) => return Err(warp::reject::custom(e)),
    };

    let question = NewQuestion {
        title,
        content,
        tags: new_question.tags,
    };

    match store.add_question(question, account_id).await {
        Ok(question) => Ok(warp::reply::json(&question)),
        Err(e) => Err(warp::reject::custom(e)),
    }
}
