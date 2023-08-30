use axum::{extract::State, Json};

use hyper::StatusCode;
use sqlx::PgPool;

use crate::{
    auth::AuthContext,
    domain::{Password, User},
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

#[derive(Debug, serde::Deserialize)]
pub(crate) struct RegisterInput {
    pub login: String,
    pub password: String,
}

pub(crate) struct NewUser {
    pub login: String,
    pub password: Password,
}

impl From<NewUser> for User {
    fn from(value: NewUser) -> Self {
        User::new(&value.login, value.password)
    }
}

impl TryFrom<RegisterInput> for NewUser {
    type Error = String;

    fn try_from(value: RegisterInput) -> Result<Self, Self::Error> {
        let password = Password::parse(&value.password)?;

        Ok(Self {
            login: value.login,
            password,
        })
    }
}

pub(crate) async fn register_handler(
    State(AppState { database }): State<AppState>,
    mut auth: AuthContext,
    Json(input): Json<RegisterInput>,
) -> Result<Json<User>, StatusCode> {
    let user = find_user_by_login(&database, &input.login)
        .await
        .map_err(|e| {
            tracing::error!("Failed to lookup user by login: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    if user.is_some() {
        return Err(StatusCode::CONFLICT);
    }

    let new_user: NewUser = input.try_into().map_err(|e| {
        // FIXME: return the error to the client
        tracing::debug!("User validation error: {e}");
        StatusCode::BAD_REQUEST
    })?;

    let user = insert_user(&database, new_user).await.map_err(|e| {
        tracing::error!("Error while saving user to database: {e}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    auth.login(&user).await.map_err(|e| {
        tracing::error!("Failed to login newly created user: {e}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(user))
}

async fn find_user_by_login(db_pool: &PgPool, login: &str) -> Result<Option<User>, String> {
    sqlx::query_as!(User, "SELECT * FROM users WHERE login = $1", login)
        .fetch_optional(db_pool)
        .await
        .map_err(|e| e.to_string())
}

async fn insert_user(pg_pool: &PgPool, new_user: NewUser) -> Result<User, String> {
    let user: User = new_user.into();
    sqlx::query_as!(
        User,
        r#"
        INSERT INTO users 
            (id, login, password_hash, role, created_at, updated_at) 
        VALUES ($1, $2, $3, $4, $5, $6) 
        RETURNING *
        "#,
        Into::<uuid::Uuid>::into(user.id),
        user.login,
        user.password_hash.as_ref(),
        user.role.as_ref(),
        user.created_at,
        user.updated_at
    )
    .fetch_one(pg_pool)
    .await
    .map_err(|e| e.to_string())
}
