use axum::{
    routing::{get, post},
    Router,
};
use deadpool_sqlite::{Config, Runtime};
use rusqlite::Result;
use std::sync::Arc;

pub mod auth;
pub mod db_helper;
pub mod routes;
pub mod messages;
pub async fn process() -> Result<()> {
    let mut cfg = Config::new("char-rs.db");
    let pool = cfg.create_pool(Runtime::Tokio1).unwrap();
    db_helper::initialize_db(&pool).await.unwrap();
    let pool = Arc::new(pool);
    let router = Router::new()
        .route("/register", post(routes::register))
        .route("/login", post(routes::login))
        .route("/test-auth", get(routes::test_auth))
        .with_state(pool.clone());
    let listener = tokio::net::TcpListener::bind("0.0.0.0:5000").await.unwrap();
    axum::serve(listener, router).await.unwrap();
    Ok(())
}
