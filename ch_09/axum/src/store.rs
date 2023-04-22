use sqlx::{
    postgres::{PgPool, PgPoolOptions, PgRow},
    Row,
};

use handle_errors::Error;

use crate::types::{
    account::{Account, AccountId},
    answer::{Answer, AnswerId, NewAnswer},
    question::{NewQuestion, Question, QuestionId},
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
        self,
        limit: Option<u32>,
        offset: u32,
    ) -> Result<Vec<Question>, Error> {
        match sqlx::query("SELECT * FROM questions LIMIT $1 OFFSET $2")
            .bind(limit.map(|l| l as i32))
            .bind(offset as i32)
            .map(|row: PgRow| Question {
                id: QuestionId(row.get("id")),
                title: row.get("title"),
                content: row.get("content"),
                tags: row.get("tags"),
            })
            .fetch_all(&self.connection)
            .await
        {
            Ok(questions) => Ok(questions),
            Err(e) => {
                tracing::event!(tracing::Level::ERROR, "{:?}", e);
                Err(Error::DatabaseQueryError(e))
            }
        }
    }

    pub async fn is_question_owner(
        &self,
        question_id: i32,
        account_id: &AccountId,
    ) -> Result<bool, Error> {
        match sqlx::query(
            "SELECT * FROM questions WHERE id = $1 and account_id = $2"
        )
        .bind(question_id)
        .bind(account_id.0)
        .fetch_optional(&self.connection)
        .await
        {
            Ok(question) => Ok(question.is_some()),
            Err(e) => {
                tracing::event!(tracing::Level::ERROR, "{:?}", e);
                Err(Error::DatabaseQueryError(e))
            }
        }
    }

    pub async fn add_question(
        self,
        title: String,
        content: String,
        tags: Vec<String>,
        account_id: AccountId,
    ) -> Result<Question, Error> {
        match sqlx::query(
            "INSERT INTO questions (title, content, tags, account_id)
                 VALUES ($1, $2, $3, $4)
                 RETURNING id, title, content, tags"
            )
            .bind(title)
            .bind(content)
            .bind(tags)
            .bind(account_id.0)
            .map(|row: PgRow| Question {
                id: QuestionId(row.get("id")),
                title: row.get("title"),
                content: row.get("content"),
                tags: row.get("tags"),
            })
            .fetch_one(&self.connection)
            .await {
                Ok(question) => Ok(question),
                Err(error) => {
                    tracing::event!(tracing::Level::ERROR, "{:?}", error);
                    Err(Error::DatabaseQueryError(error))
                },
            }
    }

    pub async fn update_question(
        self,
        question: Question,
        id: i32,
        account_id: AccountId,
    ) -> Result<Question, Error> {
        match sqlx::query(
            "UPDATE questions SET title = $1, content = $2, tags = $3
            WHERE id = $4 AND account_id = $5
            RETURNING id, title, content, tags",
        )
        .bind(question.title)
        .bind(question.content)
        .bind(question.tags)
        .bind(id)
        .bind(account_id.0)
        .map(|row: PgRow| Question {
            id: QuestionId(row.get("id")),
            title: row.get("title"),
            content: row.get("content"),
            tags: row.get("tags"),
        })
        .fetch_one(&self.connection)
        .await
        {
            Ok(question) => Ok(question),
            Err(error) => {
                tracing::event!(tracing::Level::ERROR, "{:?}", error);
                Err(Error::DatabaseQueryError(error))
            }
        }
    }

    pub async fn delete_question(
        self,
        id: i32,
        account_id: AccountId,
    ) -> Result<bool, Error> {
        match sqlx::query(
            "DELETE FROM questions WHERE id = $1 AND account_id = $2"
        )
        .bind(id)
        .bind(account_id.0)
        .execute(&self.connection)
        .await
        {
            Ok(_) => Ok(true),
            Err(e) => {
                tracing::event!(tracing::Level::ERROR, "{:?}", e);
                Err(Error::DatabaseQueryError(e))
            }
        }
    }

    pub async fn add_answer(
        self,
        question_id: QuestionId,
        content: String,
        account_id: AccountId,
    ) -> Result<Answer, Error> {
        match sqlx::query(
            "INSERT INTO answers (content, corresponding_question, account_id) VALUES ($1, $2, $3)",
        )
        .bind(content)
        .bind(question_id.0)
        .bind(account_id.0)
        .map(|row: PgRow| Answer {
            id: AnswerId(row.get("id")),
            content: row.get("content"),
            question_id: QuestionId(row.get("question_id")),
        })
        .fetch_one(&self.connection)
        .await
        {
            Ok(answer) => Ok(answer),
            Err(error) => {
                tracing::event!(
                    tracing::Level::ERROR,
                    code = error
                        .as_database_error()
                        .unwrap()
                        .code()
                        .unwrap()
                        .parse::<i32>()
                        .unwrap(),
                    db_message = error.as_database_error().unwrap().message(),
                    constraint = error.as_database_error().unwrap().constraint().unwrap()
                );
                Err(Error::DatabaseQueryError(error))
            }
        }
    }

    pub async fn add_account(
        self,
        email: String,
        password: String
    ) -> Result<bool, Error> {
        match sqlx::query(
            "INSERT INTO accounts (email, password) VALUES ($1, $2)",
        )
        .bind(email)
        .bind(password)
        .execute(&self.connection)
        .await
        {
            Ok(_) => Ok(true),
            Err(error) => {
                tracing::event!(
                    tracing::Level::ERROR,
                    code = error
                        .as_database_error()
                        .unwrap()
                        .code()
                        .unwrap()
                        .parse::<i32>()
                        .unwrap(),
                    db_message =
                        error.as_database_error().unwrap().message(),
                    constraint = error
                        .as_database_error()
                        .unwrap()
                        .constraint()
                        .unwrap()
                );
                Err(Error::DatabaseQueryError(error))
            }
        }
    }

    pub async fn get_account(
        self,
        email: String,
    ) -> Result<Account, Error> {
        match sqlx::query("SELECT * FROM accounts WHERE email = $1")
            .bind(email)
            .map(|row: PgRow| Account {
                id: Some(AccountId(row.get("id"))),
                email: row.get("email"),
                password: row.get("password"),
            })
            .fetch_one(&self.connection)
            .await
        {
            Ok(account) => Ok(account),
            Err(error) => {
                tracing::event!(tracing::Level::ERROR, "{:?}", error);
                Err(Error::DatabaseQueryError(error))
            }
        }
    }
}
