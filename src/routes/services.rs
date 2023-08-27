use axum::{extract::State, Json};
use sqlx::PgPool;

use crate::{domain::Service, startup::AppState};

#[tracing::instrument(name = "services index", skip_all)]
#[axum::debug_handler(state = crate::startup::AppState)]
pub async fn services_index(
    State(AppState { database }): State<AppState>,
) -> Result<Json<Vec<Service>>, String> {
    let services = fetch_all_services(database).await?;

    Ok(Json(services))
}

#[tracing::instrument(name = "Fetching all services", skip_all)]
async fn fetch_all_services(database: PgPool) -> Result<Vec<Service>, String> {
    sqlx::query_as!(Service, "SELECT * FROM services")
        .fetch_all(&database)
        .await
        .map_err(|e| e.to_string())
}
