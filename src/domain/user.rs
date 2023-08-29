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
