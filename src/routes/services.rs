use axum::{extract::State, Json};

use crate::{domain::Service, startup::AppState};

#[axum::debug_handler(state = crate::startup::AppState)]
pub async fn services_index(
    State(AppState { database }): State<AppState>,
) -> Result<Json<Vec<Service>>, String> {
    let services = sqlx::query_as!(Service, "SELECT * FROM services")
        .fetch_all(&database)
        .await
        .map_err(|e| e.to_string())?;

    Ok(Json(services))
}
