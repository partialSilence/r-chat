use std::collections::HashMap;
use crate::chat_api::auth::{AuthBody, AuthError, Claims, CreateUser, User};
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use deadpool_sqlite::Pool;
use jsonwebtoken::{encode, Header};
use serde::Deserialize;
use std::sync::Arc;
use crate::chat_api::db_helper::DbHelperError;
use crate::chat_api::messages::{CreateMessage, Message};

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
                (chrono::Utc::now().naive_utc() + chrono::naive::Days::new(1))
                    .and_utc().timestamp() as usize;
            let claims = crate::chat_api::auth::Claims {
                sub: val.id,
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
#[axum_macros::debug_handler]
pub async fn get_messages_thread(State(pool): State<Arc<Pool>>,claims: Claims,
Path(user_id): Path<i32>) -> (StatusCode, Json<Vec<Message>>) {
    let result = crate::chat_api::messages::
        get_messages_thread(&pool.clone(),(claims.sub, user_id)).await;
    match result {
        Ok(val) => (StatusCode::OK, Json(val)),
        Err(err) => {
            eprintln!("Error into get_messages_thread endpoint: {}", err);
            (StatusCode::OK, Json(vec![]))
        }
    }
}
pub async fn delete_message(State(pool): State<Arc<Pool>>, claims:Claims,
                            Path(message_id): Path<i64>
) -> StatusCode {
    match crate::chat_api::messages::delete_message(&pool.clone(), message_id, claims.sub).await {
        Ok(_) => StatusCode::OK,
        Err(err) => {
            eprintln!("Error into delete_message endpoint: {}", err);
            StatusCode::OK
        }
    }
}
pub async fn get_message_threads(State(pool):State<Arc<Pool>>, claims: Claims) -> (StatusCode, Json<Vec<Message>>) {
    match crate::chat_api::messages::get_message_threads(&pool.clone(), claims.sub).await {
        Ok(res) => (StatusCode::OK, Json(res)),
        Err(err) => {
            eprintln!("Error into get_message_threads endpoint: {}", err);
            (StatusCode::OK, Json(vec![]))
        }
    }
}
#[axum_macros::debug_handler]
pub async fn send_message(State(pool): State<Arc<Pool>>, claims: Claims,
                          Json(dto):Json<CreateMessage>) -> (StatusCode, Json<Option<Message>>) {
    let mut message = Message::from(dto);
    message.sender_id = claims.sub;
    match crate::chat_api::messages::create_message(&pool,message).await {
        Ok(res) => (StatusCode::CREATED, Json(Some(res))),
        Err(err) => {
            eprintln!("error into send_message: {}", err);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(None))
        },
    }

}

#[derive(Deserialize)]
pub struct Login {
    pub username: String,
    pub password: String,
}
