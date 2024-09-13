use axum::{
    routing::{get, post, delete},
    Router,
};
use deadpool_sqlite::{Config, Runtime};
use rusqlite::Result;
use std::sync::Arc;

pub mod auth;
pub mod db_helper;
pub mod messages;
pub mod routes;
pub async fn process() -> Result<()> {
    let cfg = Config::new("char-rs.db");
    let pool = cfg.create_pool(Runtime::Tokio1).unwrap();
    db_helper::initialize_db(&pool).await.unwrap();
    let pool = Arc::new(pool);
    let router = Router::new()
        .route("/register", post(routes::register))
        .route("/login", post(routes::login))
        .route("/test-auth", get(routes::test_auth))
        .route("/message", post(routes::send_message))
        .route("/message/:id", delete(routes::delete_message))
        .route("/dialogs", get(routes::get_message_threads))
        .route("/dialogs/:id", get(routes::get_messages_thread))
        .with_state(pool.clone());
    let listener = tokio::net::TcpListener::bind("0.0.0.0:5000").await.unwrap();
    axum::serve(listener, router).await.unwrap();
    Ok(())
}
