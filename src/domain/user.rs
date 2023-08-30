use argon2::{
    password_hash::{
        rand_core::OsRng, PasswordHash as ArgonPasswordHash, PasswordHasher, PasswordVerifier,
        SaltString,
    },
    Argon2,
};
use axum_login::AuthUser;
use chrono::{DateTime, Utc};
use secrecy::SecretVec;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug)]
#[repr(transparent)]
pub(crate) struct Password(String);

impl Password {
    pub(crate) fn parse(value: &str) -> Result<Self, String> {
        // FIXME: add some valdation for the password
        Ok(Self(value.to_string()))
    }

    pub(crate) fn hash(&self) -> Result<PasswordHash, String> {
        let salt = SaltString::generate(&mut OsRng);
        let hash = Argon2::default()
            .hash_password(&self.0.as_bytes(), &salt)
            .map_err(|e| e.to_string())?
            .to_string();
        Ok(PasswordHash(hash))
    }
}

#[derive(Debug, Clone, sqlx::Encode, sqlx::Decode)]
#[repr(transparent)]
pub(crate) struct PasswordHash(String);

impl PasswordHash {
    fn to_argon_hash(&self) -> ArgonPasswordHash<'_> {
        ArgonPasswordHash::new(self.0.as_str())
            .expect("failed to convert PasswordHash into ArgonPasswordHash")
    }
}

// FIXME: Why does sqlx require this?
impl From<String> for PasswordHash {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl sqlx::Type<sqlx::Postgres> for PasswordHash {
    fn type_info() -> <sqlx::Postgres as sqlx::Database>::TypeInfo {
        <String as sqlx::Type<sqlx::Postgres>>::type_info()
    }
}

#[derive(Debug, Clone, sqlx::Type, serde::Serialize, serde::Deserialize)]
#[repr(transparent)]
pub(crate) struct UserId(uuid::Uuid);

impl From<Uuid> for UserId {
    fn from(value: Uuid) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone, FromRow, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct User {
    pub id: UserId,
    #[serde(skip)]
    pub password_hash: PasswordHash,
    pub role: UserRole,
    pub login: String,
    pub updated_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

impl User {
    pub(crate) fn new(login: &str, password: Password) -> Self {
        let hash = password.hash().expect("could not hash password");
        Self {
            id: UserId(Uuid::new_v4()),
            login: login.to_string(),
            password_hash: hash,
            role: UserRole::User,
            updated_at: Utc::now(),
            created_at: Utc::now(),
        }
    }

    pub(crate) fn verify_password(&self, password: &str) -> bool {
        Argon2::default()
            .verify_password(password.as_bytes(), &self.password_hash.to_argon_hash())
            .is_ok()
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

impl AuthUser<UserId, UserRole> for User {
    fn get_id(&self) -> UserId {
        self.id.clone()
    }

    fn get_password_hash(&self) -> SecretVec<u8> {
        SecretVec::new(self.password_hash.0.clone().into())
    }

    fn get_role(&self) -> Option<UserRole> {
        Some(self.role.clone())
    }
}
