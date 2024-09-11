use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{async_trait, Json, RequestPartsExt};
use axum_extra::headers::authorization::Bearer;
use axum_extra::headers::Authorization;
use axum_extra::TypedHeader;
use jsonwebtoken::{decode, DecodingKey, EncodingKey, Validation};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use serde_json::json;
#[derive(Serialize)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub name: String,
    password_hash: String,
}
#[derive(Deserialize)]
pub struct CreateUser {
    pub username: String,
    pub name: String,
    pub password: String,
}

impl User {
    pub fn new(id: i32, username: String, name: String, password_hash: String) -> Self {
        Self {
            id,
            username,
            name,
            password_hash,
        }
    }
    pub fn get_password_hash(&self) -> String {
        self.password_hash.clone()
    }
}

impl From<CreateUser> for User {
    fn from(value: CreateUser) -> Self {
        Self {
            id: 0,
            username: value.username,
            name: value.name,
            password_hash: bcrypt::hash(value.password, 4).unwrap(),
        }
    }
}
pub struct Keys {
    pub encoding: EncodingKey,
    pub decoding: DecodingKey,
}
impl Keys {
    pub fn new(secret: &[u8]) -> Self {
        Self {
            encoding: EncodingKey::from_secret(secret),
            decoding: DecodingKey::from_secret(secret),
        }
    }
}
pub static KEYS: Lazy<Keys> = Lazy::new(|| match std::env::var("SECRET_KEY") {
    Ok(secret) => Keys::new(secret.as_bytes()),
    Err(err) => {
        eprintln!("Error when try get key from env: {}", err);
        panic!();
    }
});

pub enum AuthError {
    InvalidToken,
    WrongCredentials,
    TokenCreation,
    MissingCredentials,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AuthError::WrongCredentials => (StatusCode::UNAUTHORIZED, "Wrong credentials"),
            AuthError::TokenCreation => (StatusCode::INTERNAL_SERVER_ERROR, "Token creation error"),
            AuthError::InvalidToken => (StatusCode::BAD_REQUEST, "Invalid token"),
            AuthError::MissingCredentials => (StatusCode::BAD_REQUEST, "missing credentials"),
        };
        let body = Json(json!({
            "error": error_message
        }));
        (status, body).into_response()
    }
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}
#[async_trait]
impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| AuthError::InvalidToken)?;
        let token_data = decode::<Claims>(bearer.token(), &KEYS.decoding, &Validation::default())
            .map_err(|_| AuthError::InvalidToken)?;
        Ok(token_data.claims)
    }
}
#[derive(Debug, Serialize)]
pub struct AuthBody {
    access_token: String,
    token_type: String,
}
impl AuthBody {
    pub fn new(access_token: String) -> Self {
        Self {
            access_token,
            token_type: "Bearer".to_string(),
        }
    }
}
