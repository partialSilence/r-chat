use crate::chat_api::auth::{AuthBody, AuthError, Claims, CreateUser, User};
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use deadpool_sqlite::Pool;
use jsonwebtoken::{encode, Header};
use serde::Deserialize;
use std::sync::Arc;

#[axum_macros::debug_handler]
pub async fn register(State(pool): State<Arc<Pool>>, Json(payload): Json<CreateUser>) -> Response {
    let user = User::from(payload);
    if let Ok(id) = crate::chat_api::db_helper::create_user(user, &pool).await {
        (StatusCode::CREATED, Json(id)).into_response()
    } else {
        StatusCode::BAD_REQUEST.into_response()
    }
}
#[axum_macros::debug_handler]
pub async fn login(
    State(pool): State<Arc<Pool>>,
    Json(payload): Json<Login>,
) -> Result<Json<AuthBody>, AuthError> {
    let result = crate::chat_api::db_helper::check_user(payload, &pool).await;
    return match result {
        Ok(Some(val)) => {
            let exp =
                (chrono::Utc::now().naive_utc() + chrono::naive::Days::new(1)).timestamp() as usize;
            let claims = crate::chat_api::auth::Claims {
                sub: val.id.to_string(),
                exp,
            };
            let token = encode(
                &Header::default(),
                &claims,
                &crate::chat_api::auth::KEYS.encoding,
            )
            .map_err(|_| AuthError::TokenCreation)?;
            Ok(Json(crate::chat_api::auth::AuthBody::new(token)))
        }
        Err(err) => {
            eprintln!("error into login method: {}", err);
            Err(AuthError::WrongCredentials)
        }
        _ => {
            log::error!("User not found");
            Err(AuthError::WrongCredentials)
        }
    };
}
#[axum_macros::debug_handler]
pub async fn test_auth(claims: Claims) -> String {
    format!("Hello from {}", claims.sub)
}

#[derive(Deserialize)]
pub struct Login {
    pub username: String,
    pub password: String,
}
