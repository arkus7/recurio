use axum_login::AuthUser;
use chrono::{DateTime, Utc};
use secrecy::SecretVec;
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: uuid::Uuid,
    #[serde(skip)]
    pub password_hash: String,
    pub role: UserRole,
    pub login: String,
    pub updated_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

impl User {
    pub(crate) fn verify_password(&self, password: &str) -> bool {
        // FIXME: dummy implementation for testing purposes, use real hash checking
        return self.password_hash == password;
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, sqlx::Decode, sqlx::Encode, serde::Serialize)]
#[sqlx(rename_all = "snake_case")]
#[serde(rename_all = "camelCase")]
pub enum UserRole {
    User,
    Admin,
}

// FIXME: Consider creating a enum in database instead of storing it in `TEXT`
impl From<String> for UserRole {
    fn from(value: String) -> Self {
        match value.as_str() {
            "user" => Self::User,
            "admin" => Self::Admin,
            _ => unreachable!(),
        }
    }
}

// NOTE: Custom implementation because sqlx was looking for an enum `UserRole` in DB
impl sqlx::Type<sqlx::Postgres> for UserRole {
    fn type_info() -> <sqlx::Postgres as sqlx::Database>::TypeInfo {
        <String as sqlx::Type<sqlx::Postgres>>::type_info()
    }
}

impl AuthUser<uuid::Uuid, UserRole> for User {
    fn get_id(&self) -> uuid::Uuid {
        self.id
    }

    fn get_password_hash(&self) -> SecretVec<u8> {
        SecretVec::new(self.password_hash.clone().into())
    }

    fn get_role(&self) -> Option<UserRole> {
        Some(self.role.clone())
    }
}
