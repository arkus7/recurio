use std::collections::HashMap;
use std::sync::Arc;

use axum::async_trait;
use axum_login::axum_sessions::async_session::{Session, SessionStore};

use axum_login::{axum_sessions::SessionLayer, AuthLayer, RequireAuthorizationLayer, UserStore};
use secrecy::ExposeSecret;
use sqlx::PgPool;
use tokio::sync::RwLock;
use tracing::debug;
use uuid::Uuid;

use crate::configuration::AuthSettings;
use crate::domain::{User, UserRole};

pub(crate) type AuthContext =
    axum_login::extractors::AuthContext<uuid::Uuid, User, DatabaseUserStore, UserRole>;

pub(crate) type RequireAuth = RequireAuthorizationLayer<uuid::Uuid, User, UserRole>;

pub(crate) fn setup_auth(
    db_pool: PgPool,
    configuration: &AuthSettings,
) -> (
    AuthLayer<DatabaseUserStore, Uuid, User, UserRole>,
    SessionLayer<DatabaseUserStore>,
) {
    let user_store = DatabaseUserStore::new(db_pool.clone());
    let secret = configuration
        .session_secret
        .expose_secret()
        .as_str()
        .as_bytes();
    let auth_layer = AuthLayer::new(user_store.clone(), secret);

    let session_layer = SessionLayer::new(user_store, secret);

    (auth_layer, session_layer)
}

#[derive(Debug, Clone)]
pub(crate) struct DatabaseUserStore {
    pool: PgPool,
    query: String,
    session_store: Arc<RwLock<HashMap<String, Session>>>,
}

impl DatabaseUserStore {
    pub(crate) fn new(pool: PgPool) -> Self {
        Self {
            pool,
            query: "select * from users where id = $1".into(),
            session_store: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl UserStore<uuid::Uuid, UserRole> for DatabaseUserStore {
    type User = User;
    type Error = sqlx::error::Error;

    async fn load_user(&self, user_id: &uuid::Uuid) -> Result<Option<Self::User>, Self::Error> {
        tracing::debug!("Verifying user with ID {}", user_id);
        let user: Option<User> = sqlx::query_as(&self.query)
            .bind(user_id)
            .fetch_optional(&self.pool)
            .await?;

        tracing::debug!("Found user: {:?}", &user);

        Ok(user)
    }
}

#[async_trait]
impl SessionStore for DatabaseUserStore {
    /// Get a session from the storage backend.
    ///
    /// The input is expected to be the value of an identifying
    /// cookie. This will then be parsed by the session middleware
    /// into a session if possible
    async fn load_session(
        &self,
        cookie_value: String,
    ) -> Result<Option<Session>, axum_login::axum_sessions::async_session::Error> {
        debug!("load_session, cookie value: {}", cookie_value);
        dbg!(&self.session_store);
        let id = Session::id_from_cookie_value(&cookie_value)?;
        debug!("cookie ID: {id}");
        Ok(self
            .session_store
            .read()
            .await
            .get(&id)
            .cloned()
            .map(|s| {
                debug!("before validation, session: {:?}", s);
                s
            })
            .and_then(Session::validate)
            .map(|s| {
                debug!("after validation, session: {:?}", s);
                s
            }))
    }

    /// Store a session on the storage backend.
    ///
    /// The return value is the value of the cookie to store for the
    /// user that represents this session
    async fn store_session(
        &self,
        session: Session,
    ) -> Result<Option<String>, axum_login::axum_sessions::async_session::Error> {
        debug!("store_session, session value: {:?}", session);
        dbg!(&self.session_store);

        self.session_store
            .write()
            .await
            .insert(session.id().to_string(), session.clone());

        session.reset_data_changed();
        Ok(session.into_cookie_value())
    }

    /// Remove a session from the session store
    async fn destroy_session(
        &self,
        session: Session,
    ) -> Result<(), axum_login::axum_sessions::async_session::Error> {
        debug!("destroy_session, session value: {:?}", session);
        dbg!(&session);
        Ok(())
    }

    /// Empties the entire store, destroying all sessions
    async fn clear_store(&self) -> Result<(), axum_login::axum_sessions::async_session::Error> {
        debug!("clear_store");
        Ok(())
    }
}
