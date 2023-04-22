use sqlx::{
    Row,
    postgres::{PgPool, PgPoolOptions, PgRow}
};

use handle_errors::Error;

use crate::types::{
    answer::{Answer, AnswerId},
    question::{Question, QuestionId},
};

#[derive(Debug, Clone)]
pub struct Store {
    pub connection: PgPool,
}

impl Store {
    pub async fn new(url: &str) -> Self {
        let pool = match PgPoolOptions::new()
            .max_connections(5)
            .connect(url)
            .await
        {
            Ok(pool) => pool,
            Err(e) => panic!("Couldn't establish DB connection: {}", e),
        };

        Store { connection: pool }
    }

    pub async fn get_questions(
        &self,
        limit: Option<u32>,
        offset: u32,
    ) -> Result<Vec<Question>, Error> {
        match sqlx::query("SELECT * from questions LIMIT $1 OFFSET $2")
            .bind(limit.map(|l| l as i32))
            .bind(offset as i32)
            .map(|row: PgRow| Question {
                id: QuestionId(row.get("id")),
                title: row.get("title"),
                content: row.get("content"),
                tags: row.get("tags"),
            })
            .fetch_all(&self.connection)
            .await {
                Ok(questions) => Ok(questions),
                Err(e) => {
                    tracing::event!(tracing::Level::ERROR, "{:?}", e);
                    Err(Error::DatabaseQueryError)
            }
        }
    }

    pub async fn add_question(
        &self,
        title: String,
        content: String,
        tags: Vec<String>
    ) -> Result<Question, Error> {
        match sqlx::query(
            "INSERT INTO questions (title, content, tags)
                 VALUES ($1, $2, $3)
                 RETURNING id, title, content, tags",
        )
        .bind(title)
        .bind(content)
        .bind(tags)
        .map(|row: PgRow| Question {
            id: QuestionId(row.get("id")),
            title: row.get("title"),
            content: row.get("content"),
            tags: row.get("tags"),
        })
        .fetch_one(&self.connection)
        .await {
            Ok(question) => Ok(question),
            Err(e) => {
                tracing::event!(tracing::Level::ERROR, "{:?}", e);
                Err(Error::DatabaseQueryError)
            }
        }
    }

    pub async fn update_question(
        &self,
        question: Question,
        id: i32,
    ) -> Result<Question, Error> {
        match sqlx::query(
            "UPDATE questions SET title = $1, content = $2, tags = $3
            WHERE id = $4
            RETURNING id, title, content, tags",
        )
        .bind(question.title)
        .bind(question.content)
        .bind(question.tags)
        .bind(id)
        .map(|row: PgRow| Question {
            id: QuestionId(row.get("id")),
            title: row.get("title"),
            content: row.get("content"),
            tags: row.get("tags"),
        })
        .fetch_one(&self.connection)
        .await {
            Ok(question) => Ok(question),
            Err(e) => {
                tracing::event!(tracing::Level::ERROR, "{:?}", e);
                Err(Error::DatabaseQueryError)
            }
        }
    }

    pub async fn delete_question(&self, id: i32) -> Result<bool, Error> {
        match sqlx::query("DELETE FROM questions WHERE id = $1")
        .bind(id)
        .execute(&self.connection)
        .await {
            Ok(_) => Ok(true),
            Err(e) => {
                tracing::event!(tracing::Level::ERROR, "{:?}", e);
                Err(Error::DatabaseQueryError)
            }
        }
    }

    pub async fn add_answer(
        &self,
        question_id: QuestionId,
        content: String
    ) -> Result<Answer, Error> {
        match sqlx::query(
            "INSERT INTO answers (content, question_id) VALUES ($1, $2)",
        )
        .bind(content)
        .bind(question_id.0)
        .map(|row: PgRow| Answer {
            id: AnswerId(row.get("id")),
            content: row.get("content"),
            question_id: QuestionId(row.get("question_id")),
        })
        .fetch_one(&self.connection)
        .await {
            Ok(answer) => Ok(answer),
            Err(e) => {
                tracing::event!(tracing::Level::ERROR, "{:?}", e);
                Err(Error::DatabaseQueryError)
            }
        }
    }
}
