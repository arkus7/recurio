use axum::{extract::State, Json};


use crate::{
    domain::{Service, ServiceName},
    startup::AppState,
};

#[axum::debug_handler(state = crate::startup::AppState)]
pub async fn services_index(State(AppState { database }): State<AppState>) -> Json<Vec<Service>> {
    let test: Result<String, sqlx::Error> = sqlx::query_scalar("select 'hello world from pg'")
        .fetch_one(&database)
        .await;

    dbg!(&test);

    let services = vec![Service {
        id: 1,
        name: ServiceName::parse("Netflix".into()).unwrap(),
        image_url: test.ok(),
        owner_id: None,
    }];

    Json(services)
}
