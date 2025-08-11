use axum_login::{AuthManagerLayer, AuthManagerLayerBuilder, AuthSession, AuthnBackend, UserId as BackendUserId};
use axum_login::tower_sessions::{SessionManagerLayer, MemoryStore};
use sqlx::PgPool;

use crate::configuration::AuthSettings;
use crate::domain::User;

pub(crate) type AuthContext = AuthSession<Backend>;
pub(crate) type AuthLayer = AuthManagerLayer<Backend, MemoryStore>;

#[derive(Debug)]
pub struct BackendError(pub String);

impl core::fmt::Display for BackendError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for BackendError {}

#[derive(Clone)]
pub(crate) struct Backend {
    pub pool: PgPool,
}

#[derive(Clone)]
pub(crate) struct Credentials {
    pub login: String,
    pub password: String,
}

impl AuthnBackend for Backend {
    type User = User;
    type Credentials = Credentials;
    type Error = crate::auth::BackendError;

    fn authenticate(
        &self,
        creds: Self::Credentials,
    ) -> impl core::future::Future<Output = Result<Option<Self::User>, Self::Error>> + Send {
        let pool = self.pool.clone();
        async move {
            let user = sqlx::query_as!(User, "SELECT * FROM users WHERE login = $1", creds.login)
                .fetch_optional(&pool)
                .await
                .map_err(|e| BackendError(e.to_string()))?;
            Ok(user.filter(|u| u.verify_password(&creds.password)))
        }
    }

    fn get_user(
        &self,
        user_id: &BackendUserId<Self>,
    ) -> impl core::future::Future<Output = Result<Option<Self::User>, Self::Error>> + Send {
        let pool = self.pool.clone();
        let uuid: uuid::Uuid = user_id.clone().into();
        async move {
            let user = sqlx::query_as!(User, "SELECT * FROM users WHERE id = $1", uuid)
                .fetch_optional(&pool)
                .await
                .map_err(|e| BackendError(e.to_string()))?;
            Ok(user)
        }
    }
}

pub(crate) async fn setup_auth(
    db_pool: PgPool,
    _configuration: &AuthSettings,
) -> AuthLayer {
    let backend = Backend { pool: db_pool.clone() };

    // Build session manager layer using in-memory store for demo purposes
    let session_layer = SessionManagerLayer::new(MemoryStore::default())
        .with_name("recurio.sid");

    AuthManagerLayerBuilder::new(backend, session_layer)
        .build()
}

// #[async_trait]
// impl SessionStore for DatabaseUserStore {
//     /// Get a session from the storage backend.
//     ///
//     /// The input is expected to be the value of an identifying
//     /// cookie. This will then be parsed by the session middleware
//     /// into a session if possible
//     async fn load_session(
//         &self,
//         cookie_value: String,
//     ) -> Result<Option<Session>, axum_login::axum_sessions::async_session::Error> {
//         debug!("load_session, cookie value: {}", cookie_value);
//         dbg!("session_store", &self.session_store);
//         let id = Session::id_from_cookie_value(&cookie_value)?;
//         debug!("cookie ID: {id}");
//         Ok(self
//             .session_store
//             .read()
//             .await
//             .get(&id)
//             .cloned()
//             .map(|s| {
//                 debug!("before validation, session: {:?}", s);
//                 s
//             })
//             .and_then(Session::validate)
//             .map(|s| {
//                 debug!("after validation, session: {:?}", s);
//                 s
//             }))
//     }
//
//     /// Store a session on the storage backend.
//     ///
//     /// The return value is the value of the cookie to store for the
//     /// user that represents this session
//     async fn store_session(
//         &self,
//         session: Session,
//     ) -> Result<Option<String>, axum_login::axum_sessions::async_session::Error> {
//         debug!("store_session, session value: {:?}", session);
//         dbg!("session_store before store", &self.session_store);
//
//         self.session_store
//             .write()
//             .await
//             .insert(session.id().to_string(), session.clone());
//
//         dbg!("session_store after store", &self.session_store);
//         session.reset_data_changed();
//         Ok(session.into_cookie_value())
//     }
//
//     /// Remove a session from the session store
//     async fn destroy_session(
//         &self,
//         session: Session,
//     ) -> Result<(), axum_login::axum_sessions::async_session::Error> {
//         debug!("destroy_session");
//         dbg!("session to destroy", &session);
//         self.session_store.write().await.remove(session.id());
//         Ok(())
//     }
//
//     /// Empties the entire store, destroying all sessions
//     async fn clear_store(&self) -> Result<(), axum_login::axum_sessions::async_session::Error> {
//         debug!("clear_store");
//         Ok(())
//     }
// }
