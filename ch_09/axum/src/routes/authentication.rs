use std::future;
use axum::{
    response::IntoResponse
};
use chrono::{Duration, Utc};
use argon2::{self, Config};
use rand::Rng;

use crate::{
    store::Store,
    types::account::{Account, AccountId, Session}
};

pub async fn register(
    State(store): State<Arc<Store>>,
    Json(Account { id, email, password }): Json<Account>,
) -> impl IntoResponse {
    let hashed_password = hash_password(password.as_bytes());
    match store.add_account(email, hashed_password).await {
        Ok(_) => Ok("Account added"),
        Err(e) => Err(e),
    }
}

pub async fn login(
    State(store): State<Arc<Store>>,
    Json(Account { id, email, password }): Json<Account>,
) -> impl IntoResponse {
    match store.get_account(email).await {
        Ok(account) => match verify_password(&password, password.as_bytes()) {
            Ok(verified) if verified
                => Ok(Json(&issue_token(id.expect("id not found")))),
            Ok(verified) => Err(handle_errors::Error::WrongPassword),
            Err(e) => Err(handle_errors::Error::ArgonLibraryError(e)),
        },
        Err(e) => Err(e),
    }
}

pub fn verify_token(
    token: String,
) -> Result<Session, handle_errors::Error> {
    let token = paseto::tokens::validate_local_token(
        &token,
        None,
        &"RANDOM WORDS WINTER MACINTOSH PC".as_bytes(),
        &paseto::tokens::TimeBackend::Chrono,
    )
    .map_err(|_| handle_errors::Error::CannotDecryptToken)?;

    serde_json::from_value::<Session>(token)
        .map_err(|_| handle_errors::Error::CannotDecryptToken)
}

fn hash_password(password: &[u8]) -> String {
    let salt = rand::thread_rng().gen::<[u8; 32]>();
    let config = Config::default();
    argon2::hash_encoded(password, &salt, &config).unwrap()
}

fn verify_password(
    hash: &str,
    password: &[u8],
) -> Result<bool, argon2::Error> {
    argon2::verify_encoded(hash, password)
}

fn issue_token(account_id: AccountId) -> String {
    let current_date_time = Utc::now();
    let dt = current_date_time + Duration::days(1);

    paseto::tokens::PasetoBuilder::new()
        .set_encryption_key(&Vec::from(
            "RANDOM WORDS WINTER MACINTOSH PC".as_bytes(),
        ))
        .set_expiration(&dt)
        .set_not_before(&Utc::now())
        .set_claim("account_id", serde_json::json!(account_id))
        .build()
        .expect("Failed to construct paseto token w/ builder!")
}

pub fn auth(
) -> impl Filter<Extract = (Session,), Error = warp::Rejection> + Clone {
    warp::header::<String>("Authorization").and_then(|token: String| {
        let token = match verify_token(token) {
            Ok(t) => t,
            Err(_) => return future::ready(Err(warp::reject::reject())),
        };

        future::ready(Ok(token))
    })
}
