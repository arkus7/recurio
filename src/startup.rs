use axum::{routing::get, Router};

use crate::routes::{health_check, services_router};

pub fn app() -> Router {
    Router::new()
        .nest("/api", api_router())
        .route("/", get(|| async { "Hello, World!" }))
        .route("/health", get(health_check))
}

fn api_router() -> Router {
    Router::new().nest("/services", services_router())
}
