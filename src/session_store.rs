use async_trait::async_trait;
use axum_login::axum_sessions::async_session::{Result, Session, SessionStore};
use chrono::{DateTime, Utc};
use sqlx::{Executor, PgPool};

#[derive(Debug, Clone)]
pub(crate) struct PostgresSessionStore {
    database: PgPool,
    table_name: String,
}

impl PostgresSessionStore {
    pub fn new(database: PgPool, table_name: &str) -> Self {
        Self {
            database,
            table_name: table_name.into(),
        }
    }

    pub async fn migrate(&self) -> std::result::Result<(), sqlx::Error> {
        tracing::info!("Migrating sessions table: {}", self.table_name);

        let query = format!(
            r#"
            CREATE TABLE IF NOT EXISTS {table_name} (
                "id" VARCHAR NOT NULL PRIMARY KEY,
                "expires" TIMESTAMPTZ NULL,
                "session" TEXT NOT NULL
            )
            "#,
            table_name = self.table_name
        );

        self.database.execute(&*query).await?;

        Ok(())
    }
}

#[async_trait]
impl SessionStore for PostgresSessionStore {
    async fn load_session(&self, cookie_value: String) -> Result<Option<Session>> {
        let id = Session::id_from_cookie_value(&cookie_value)?;

        let result: Option<(String,)> = sqlx::query_as(&*format!(
            "SELECT session FROM {table_name} WHERE id = $1 AND (expires IS NULL OR expires > $2)",
            table_name = self.table_name
        ))
        .bind(&id)
        .bind(Utc::now())
        .fetch_optional(&self.database)
        .await?;

        Ok(result
            .map(|(session,)| serde_json::from_str(&session))
            .transpose()?)
    }

    async fn store_session(&self, session: Session) -> Result<Option<String>> {
        let id = session.id();
        let string = serde_json::to_string(&session)?;

        sqlx::query(&*format!(
            r#"
            INSERT INTO {table_name}
              (id, session, expires) SELECT $1, $2, $3
            ON CONFLICT(id) DO UPDATE SET
              expires = EXCLUDED.expires,
              session = EXCLUDED.session
            "#,
            table_name = self.table_name
        ))
        .bind(&id)
        .bind(&string)
        .bind(&session.expiry())
        .execute(&self.database)
        .await?;

        Ok(session.into_cookie_value())
    }

    async fn destroy_session(&self, session: Session) -> Result {
        let id = session.id();
        sqlx::query(&*format!(
            "DELETE FROM {table_name} WHERE id = $1",
            table_name = self.table_name
        ))
        .bind(&id)
        .execute(&self.database)
        .await?;

        Ok(())
    }

    async fn clear_store(&self) -> Result {
        sqlx::query(&*format!(
            "TRUNCATE {table_name}",
            table_name = self.table_name
        ))
        .execute(&self.database)
        .await?;

        Ok(())
    }
}
