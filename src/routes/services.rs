use axum::{response::IntoResponse, routing::get, Json, Router};

use crate::domain::{Service, ServiceName};

pub fn services_router() -> Router {
    Router::new().route("/", get(services_index))
}

async fn services_index() -> impl IntoResponse {
    let services = vec![Service {
        id: 1,
        name: ServiceName::parse("Netflix".into()).unwrap(),
        image_url: None,
        owner_id: None,
    }];

    Json(services)
}
