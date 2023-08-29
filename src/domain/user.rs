use axum_login::AuthUser;
use secrecy::SecretVec;
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow)]
pub struct User {
    pub id: uuid::Uuid,
    pub password_hash: String,
    pub role: UserRole,
    pub login: String,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, sqlx::Decode, sqlx::Encode)]
#[sqlx(rename_all = "snake_case")]
pub enum UserRole {
    User,
    Admin,
}

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
