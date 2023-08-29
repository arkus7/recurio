

use axum::{extract::State, Json};

use hyper::StatusCode;
use sqlx::PgPool;

use crate::{
    auth::{AuthContext},
    domain::{User},
    startup::AppState,
};

#[derive(Debug, serde::Deserialize)]
pub struct LoginInput {
    pub login: String,
    pub password: String,
}

#[axum::debug_handler(state = crate::startup::AppState)]
pub(crate) async fn login_handler(
    State(AppState { database }): State<AppState>,
    mut auth: AuthContext,
    Json(input): Json<LoginInput>,
) -> Result<Json<User>, StatusCode> {
    let user = find_user_by_login(&database, &input.login)
        .await
        .map_err(|e| {
            tracing::error!(e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::UNAUTHORIZED)?;

    if !user.verify_password(&input.password) {
        return Err(StatusCode::UNAUTHORIZED);
    }

    auth.login(&user).await.unwrap();

    dbg!(&auth.current_user);

    Ok(Json(user))
}

async fn find_user_by_login(db_pool: &PgPool, login: &str) -> Result<Option<User>, String> {
    sqlx::query_as!(User, "SELECT * FROM users WHERE login = $1", login)
        .fetch_optional(db_pool)
        .await
        .map_err(|e| e.to_string())
}
