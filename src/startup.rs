use axum::{routing::get, Router};

use crate::routes::health_check;

pub fn app() -> Router {
    Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/health", get(health_check))
}
