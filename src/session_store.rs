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

    pub async fn count(&self) -> sqlx::Result<i64> {
        let (count,) = sqlx::query_as(&*format!(
            "SELECT COUNT(*) FROM {table_name}",
            table_name = self.table_name
        ))
        .fetch_one(&self.database)
        .await?;

        Ok(count)
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

#[cfg(test)]
mod tests {
    use crate::configuration::get_configuration;

    use super::*;
    use std::time::Duration;

    async fn create_db_for_tests() -> PgPool {
        let config = get_configuration().expect("Failed to read configuration");
        let connection_opts = config.database.without_db();

        let pool = PgPool::connect_lazy_with(connection_opts.clone());

        let db_name = uuid::Uuid::new_v4().to_string().replace('-', "");
        let db_name = format!("test_sessions_{db_name}");
        sqlx::query(&*format!("CREATE DATABASE {db_name}"))
            .execute(&pool)
            .await
            .expect("Failed to create db for tests");

        let connection_opts = connection_opts.database(&db_name);
        PgPool::connect_lazy_with(connection_opts)
    }

    async fn test_store() -> PostgresSessionStore {
        let database = create_db_for_tests().await;
        let store = PostgresSessionStore::new(database, "async_sessions");

        store
            .migrate()
            .await
            .expect("migrating a PostgresSessionStore");

        store.clear_store().await.expect("clearing");

        store
    }

    #[tokio::test]
    async fn creating_a_new_session_with_no_expiry() -> Result {
        let store = test_store().await;
        let mut session = Session::new();
        session.insert("key", "value")?;
        let cloned = session.clone();
        let cookie_value = store.store_session(session).await?.unwrap();

        let (id, expires, serialized, count): (String, Option<DateTime<Utc>>, String, i64) =
            sqlx::query_as("select id, expires, session, (select count(*) from async_sessions) from async_sessions")
                .fetch_one(&store.database)
                .await?;

        assert_eq!(1, count);
        assert_eq!(id, cloned.id());
        assert_eq!(expires, None);

        let deserialized_session: Session = serde_json::from_str(&serialized)?;
        assert_eq!(cloned.id(), deserialized_session.id());
        assert_eq!("value", &deserialized_session.get::<String>("key").unwrap());

        let loaded_session = store.load_session(cookie_value).await?.unwrap();
        assert_eq!(cloned.id(), loaded_session.id());
        assert_eq!("value", &loaded_session.get::<String>("key").unwrap());

        assert!(!loaded_session.is_expired());
        Ok(())
    }

    #[tokio::test]
    async fn updating_a_session() -> Result {
        let store = test_store().await;
        let mut session = Session::new();
        let original_id = session.id().to_owned();

        session.insert("key", "value")?;
        let cookie_value = store.store_session(session).await?.unwrap();

        let mut session = store.load_session(cookie_value.clone()).await?.unwrap();
        session.insert("key", "other value")?;
        assert_eq!(None, store.store_session(session).await?);

        let session = store.load_session(cookie_value.clone()).await?.unwrap();
        assert_eq!(session.get::<String>("key").unwrap(), "other value");

        let (id, count): (String, i64) =
            sqlx::query_as("select id, (select count(*) from async_sessions) from async_sessions")
                .fetch_one(&store.database)
                .await?;

        assert_eq!(1, count);
        assert_eq!(original_id, id);

        Ok(())
    }

    #[tokio::test]
    async fn updating_a_session_extending_expiry() -> Result {
        let store = test_store().await;
        let mut session = Session::new();
        session.expire_in(Duration::from_secs(10));
        let original_id = session.id().to_owned();
        let original_expires = session.expiry().unwrap().clone();
        let cookie_value = store.store_session(session).await?.unwrap();

        let mut session = store.load_session(cookie_value.clone()).await?.unwrap();
        assert_eq!(session.expiry().unwrap(), &original_expires);
        session.expire_in(Duration::from_secs(20));
        let new_expires = session.expiry().unwrap().clone();
        store.store_session(session).await?;

        let session = store.load_session(cookie_value.clone()).await?.unwrap();
        assert_eq!(session.expiry().unwrap(), &new_expires);

        let (id, expires, count): (String, DateTime<Utc>, i64) = sqlx::query_as(
            "select id, expires, (select count(*) from async_sessions) from async_sessions",
        )
        .fetch_one(&store.database)
        .await?;

        assert_eq!(1, count);
        assert_eq!(expires.timestamp(), new_expires.timestamp());
        assert_eq!(original_id, id);

        Ok(())
    }

    #[tokio::test]
    async fn creating_a_new_session_with_expiry() -> Result {
        let store = test_store().await;
        let mut session = Session::new();
        session.expire_in(Duration::from_secs(1));
        session.insert("key", "value")?;
        let cloned = session.clone();

        let cookie_value = store.store_session(session).await?.unwrap();

        let (id, expires, serialized, count): (String, Option<DateTime<Utc>>, String, i64) =
            sqlx::query_as("select id, expires, session, (select count(*) from async_sessions) from async_sessions")
                .fetch_one(&store.database)
                .await?;

        assert_eq!(1, count);
        assert_eq!(id, cloned.id());
        assert!(expires.unwrap() > Utc::now());

        let deserialized_session: Session = serde_json::from_str(&serialized)?;
        assert_eq!(cloned.id(), deserialized_session.id());
        assert_eq!("value", &deserialized_session.get::<String>("key").unwrap());

        let loaded_session = store.load_session(cookie_value.clone()).await?.unwrap();
        assert_eq!(cloned.id(), loaded_session.id());
        assert_eq!("value", &loaded_session.get::<String>("key").unwrap());

        assert!(!loaded_session.is_expired());

        tokio::time::sleep(Duration::from_secs(1)).await;
        assert_eq!(None, store.load_session(cookie_value).await?);

        Ok(())
    }

    #[tokio::test]
    async fn destroying_a_single_session() -> Result {
        let store = test_store().await;
        for _ in 0..3i8 {
            store.store_session(Session::new()).await?;
        }

        let cookie = store.store_session(Session::new()).await?.unwrap();
        assert_eq!(4, store.count().await?);
        let session = store.load_session(cookie.clone()).await?.unwrap();
        store.destroy_session(session.clone()).await.unwrap();
        assert_eq!(None, store.load_session(cookie).await?);
        assert_eq!(3, store.count().await?);

        // // attempting to destroy the session again is not an error
        assert!(store.destroy_session(session).await.is_ok());
        Ok(())
    }

    #[tokio::test]
    async fn clearing_the_whole_store() -> Result {
        let store = test_store().await;
        for _ in 0..3i8 {
            store.store_session(Session::new()).await?;
        }

        assert_eq!(3, store.count().await?);
        store.clear_store().await.unwrap();
        assert_eq!(0, store.count().await?);

        Ok(())
    }
}
