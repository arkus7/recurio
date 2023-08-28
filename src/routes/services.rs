use axum::{extract::State, Json};
use hyper::StatusCode;
use sqlx::PgPool;

use crate::{
    domain::{NewService, Service, ServiceName},
    startup::AppState,
};
use uuid::Uuid;

#[derive(Debug, serde::Deserialize)]
pub struct CreateService {
    name: String,
}

impl TryFrom<CreateService> for NewService {
    type Error = String;

    fn try_from(value: CreateService) -> Result<Self, Self::Error> {
        let name = ServiceName::parse(value.name)?;

        Ok(Self { name })
    }
}

#[tracing::instrument(name = "services index", skip_all)]
#[axum::debug_handler(state = crate::startup::AppState)]
pub async fn services_index(
    State(AppState { database }): State<AppState>,
) -> Result<Json<Vec<Service>>, String> {
    let services = fetch_all_services(database).await?;

    Ok(Json(services))
}

#[tracing::instrument(name = "Create service", skip_all, fields(service_name = %input.name))]
#[axum::debug_handler(state = crate::startup::AppState)]
pub async fn create_service(
    State(AppState { database }): State<AppState>,
    Json(input): Json<CreateService>,
) -> Result<Json<Service>, (StatusCode, String)> {
    let new_service: NewService = input.try_into().map_err(|e| (StatusCode::BAD_REQUEST, e))?;

    let service = insert_service(database, new_service)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(service))
}

#[tracing::instrument(name = "Fetching all services", skip_all)]
async fn fetch_all_services(database: PgPool) -> Result<Vec<Service>, String> {
    sqlx::query_as!(Service, "SELECT * FROM services")
        .fetch_all(&database)
        .await
        .map_err(|e| e.to_string())
}

#[tracing::instrument(name = "Save service in the database", skip_all)]
async fn insert_service(database: PgPool, service: NewService) -> Result<Service, String> {
    sqlx::query_as!(
        Service,
        r#"
        INSERT INTO services(id, name)
        VALUES ($1, $2)
        RETURNING *
        "#,
        Uuid::new_v4(),
        service.name.as_ref()
    )
    .fetch_one(&database)
    .await
    .map_err(|e| e.to_string())
}
