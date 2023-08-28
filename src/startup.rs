use std::{net::TcpListener, time::Duration};

use axum::{routing::get, Router};
use sqlx::{postgres::PgPoolOptions, PgPool};

use crate::{
    configuration::{DatabaseSettings, Settings},
    routes::{create_service, health_check, services_index},
};

pub struct Application {
    port: u16,
    router: axum::Router,
}

#[derive(Clone)]
pub struct AppState {
    pub database: PgPool,
}

impl Application {
    pub async fn build(configuration: Settings) -> Result<Self, std::io::Error> {
        let connection_pool = get_connection_pool(&configuration.database);

        let address = format!(
            "{}:{}",
            configuration.application.host, configuration.application.port
        );

        let listener = TcpListener::bind(address)?;
        let port = listener.local_addr().unwrap().port();

        let state = AppState {
            database: connection_pool,
        };

        let router = app(state);

        Ok(Self { port, router })
    }

    pub fn router(&self) -> Router {
        self.router.clone()
    }

    pub fn port(&self) -> u16 {
        self.port
    }
}

pub fn app(state: AppState) -> Router {
    Router::new()
        .nest("/api", api_router(state))
        .route("/", get(|| async { "Hello, World!" }))
        .route("/health", get(health_check))
}

fn api_router(state: AppState) -> Router {
    Router::new()
        .route("/services", get(services_index).post(create_service))
        .with_state(state)
}

fn get_connection_pool(configuration: &DatabaseSettings) -> PgPool {
    PgPoolOptions::new()
        .acquire_timeout(Duration::from_secs(3))
        .connect_lazy_with(configuration.with_db())
}
