use std::{env, future};
use argon2::{self, Config};
use axum::{
    extract::State,
    response::{IntoResponse, Json},
};
use chrono::{Duration, Utc};
use rand::Rng;

use crate::store::Store;
use crate::types::account::{Account, AccountId, Session};

pub async fn register(State(store): State<Arc<Store>>, Json(account): Json<Account>)
    -> impl IntoResponse
{
    let hashed_password = hash_password(account.password.as_bytes());

    let account = Account {
        id: account.id,
        email: account.email,
        password: hashed_password,
    };

    match store.add_account(account).await {
        Ok(_) => Ok(Json(&"Account added".to_string())),
        Err(e) => Err(e),
    }
}

pub async fn login(State(store): State<Arc<Store>>, Json(login): Json<Account>)
    -> impl IntoResponse
{
    match store.get_account(login.email).await {
        Ok(account) => match verify_password(&account.password, login.password.as_bytes()) {
            Ok(verified) => {
                if verified {
                    Ok(Json(&issue_token(account.id.expect("id not found"))))
                } else {
                    Err(handle_errors::Error::WrongPassword)
                }
            }
            Err(e) => Err(handle_errors::Error::ArgonLibraryError(e)),
        },
        Err(_) => Err(handle_errors::Error::WrongPassword),
    }
}

fn hash_password(password: &[u8]) -> String {
    let salt = rand::thread_rng().gen::<[u8; 32]>();
    let config = Config::default();
    argon2::hash_encoded(password, &salt, &config).unwrap()
}

fn verify_password(hash: &str, password: &[u8]) -> Result<bool, argon2::Error> {
    argon2::verify_encoded(hash, password)
}

pub fn verify_token(token: String) -> Result<Session, handle_errors::Error> {
    let key = env::var("PASETO_KEY").unwrap();
    let token = paseto::tokens::validate_local_token(
        &token,
        None,
        key.as_bytes(),
        &paseto::tokens::TimeBackend::Chrono,
    )
    .map_err(|_| handle_errors::Error::CannotDecryptToken)?;
    serde_json::from_value::<Session>(token).map_err(|_| handle_errors::Error::CannotDecryptToken)
}

fn issue_token(account_id: AccountId) -> String {
    let key = env::var("PASETO_KEY").unwrap();

    let current_date_time = Utc::now();
    let dt = current_date_time + Duration::days(1);

    paseto::tokens::PasetoBuilder::new()
        .set_encryption_key(&Vec::from(key.as_bytes()))
        .set_expiration(&dt)
        .set_claim("account_id", serde_json::json!(account_id))
        .build()
        .expect("Failed to construct paseto token w/ builder!")
}

pub fn auth() -> impl Filter<Extract = (Session,), Error = warp::Rejection> + Clone {
    warp::header::<String>("Authorization").and_then(|token: String| {
        let token = match verify_token(token) {
            Ok(t) => t,
            Err(_) => {
                return future::ready(Err(handle_errors::Error::Unauthorized))
            }
        };

        future::ready(Ok(token))
    })
}

#[cfg(test)]
mod authentication_tests {
    use super::{auth, env, issue_token, AccountId};

    #[tokio::test]
    async fn post_questions_auth() {
        env::set_var("PASETO_KEY", "RANDOM WORDS WINTER MACINTOSH PC");
        let token = issue_token(AccountId(3));

        let filter = auth();

        let res = warp::test::request()
            .header("Authorization", token)
            .filter(&filter);

        assert_eq!(res.await.unwrap().account_id, AccountId(3));
    }
}
